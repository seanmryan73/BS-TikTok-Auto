use super::token_store::TokenData;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

const AUTH_ENDPOINT: &str = "https://www.tiktok.com/v2/auth/authorize/";
const TOKEN_ENDPOINT: &str = "https://open.tiktokapis.com/oauth/access_token/";
const REFRESH_ENDPOINT: &str = "https://open.tiktokapis.com/oauth/token/";
const REDIRECT_URI: &str = "http://localhost:8080/callback/";

#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
    refresh_token: String,
    refresh_expires_in: u64,
    #[serde(default)]
    open_id: String,
}

/// Returns `(code_verifier, code_challenge)` for PKCE S256.
pub fn gen_pkce_pair() -> (String, String) {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    // Base64url-encode the random bytes → code_verifier (43 URL-safe chars, no padding)
    let verifier = URL_SAFE_NO_PAD.encode(bytes);
    // SHA-256 of the verifier string's bytes, then base64url-encode → code_challenge
    let hash = Sha256::digest(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(hash.as_slice());
    (verifier, challenge)
}

pub fn gen_state() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:016x}", nanos)
}

pub fn build_auth_url(client_key: &str, state: &str, code_challenge: &str) -> String {
    format!(
        "{}?client_key={}&scope=video.upload%2Cvideo.publish\
         &response_type=code\
         &redirect_uri=http%3A%2F%2Flocalhost%3A8080%2Fcallback%2F\
         &state={}\
         &code_challenge={}\
         &code_challenge_method=S256",
        AUTH_ENDPOINT, client_key, state, code_challenge
    )
}

pub fn exchange_code(
    client_key: &str,
    client_secret: &str,
    code: &str,
    code_verifier: &str,
) -> Result<TokenData, String> {
    let resp = ureq::post(TOKEN_ENDPOINT)
        .send_form(&[
            ("client_key", client_key),
            ("client_secret", client_secret),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", REDIRECT_URI),
            ("code_verifier", code_verifier),
        ])
        .map_err(|e| format!("Token request failed: {e}"))?;

    let tr: TokenResponse = serde_json::from_reader(resp.into_reader())
        .map_err(|e| format!("Token parse failed: {e}"))?;

    let now = unix_now();
    Ok(TokenData {
        access_token: tr.access_token,
        refresh_token: tr.refresh_token,
        expires_at: now + tr.expires_in,
        refresh_expires_at: now + tr.refresh_expires_in,
        open_id: tr.open_id,
    })
}

pub fn refresh_token(
    client_key: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<TokenData, String> {
    let resp = ureq::post(REFRESH_ENDPOINT)
        .send_form(&[
            ("client_key", client_key),
            ("client_secret", client_secret),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ])
        .map_err(|e| format!("Refresh request failed: {e}"))?;

    let tr: TokenResponse = serde_json::from_reader(resp.into_reader())
        .map_err(|e| format!("Refresh parse failed: {e}"))?;

    let now = unix_now();
    Ok(TokenData {
        access_token: tr.access_token,
        refresh_token: tr.refresh_token,
        expires_at: now + tr.expires_in,
        refresh_expires_at: now + tr.refresh_expires_in,
        open_id: tr.open_id,
    })
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
