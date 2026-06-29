use super::{tiktok_auth, AuthResult};
use std::collections::HashMap;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tiny_http::{Header, Response, Server};

/// Spawns the OAuth flow on a background thread.
/// The thread binds :8080, waits for TikTok's callback, exchanges the code,
/// and sends the result via `tx`. Send anything on `stop_rx` to abort early.
pub fn start(
    client_key: String,
    client_secret: String,
    expected_state: String,
    code_verifier: String,
    tx: mpsc::Sender<AuthResult>,
    stop_rx: mpsc::Receiver<()>,
) {
    std::thread::spawn(move || {
        run(client_key, client_secret, expected_state, code_verifier, tx, stop_rx)
    });
}

fn run(
    client_key: String,
    client_secret: String,
    expected_state: String,
    code_verifier: String,
    tx: mpsc::Sender<AuthResult>,
    stop_rx: mpsc::Receiver<()>,
) {
    let server = match Server::http("127.0.0.1:8080") {
        Ok(s) => s,
        Err(e) => {
            let _ = tx.send(AuthResult::Error(format!("Cannot bind :8080 — {e}")));
            return;
        }
    };

    let deadline = Instant::now() + Duration::from_secs(120);

    let request = loop {
        if stop_rx.try_recv().is_ok() {
            return;
        }
        if Instant::now() >= deadline {
            let _ = tx.send(AuthResult::Error(
                "Login timed out (2 min). Please try again.".into(),
            ));
            return;
        }
        match server.try_recv() {
            Ok(Some(req)) => break req,
            Ok(None) => std::thread::sleep(Duration::from_millis(200)),
            Err(e) => {
                let _ = tx.send(AuthResult::Error(format!("Callback server error: {e}")));
                return;
            }
        }
    };

    // Parse query string from the callback URL
    let url = request.url().to_owned();
    let query = url.splitn(2, '?').nth(1).unwrap_or("");
    let params: HashMap<&str, String> = query
        .split('&')
        .filter_map(|pair| {
            let mut it = pair.splitn(2, '=');
            let k = it.next()?;
            let v = it.next().unwrap_or("").replace('+', " ");
            Some((k, v))
        })
        .collect();

    // Respond to the browser immediately so it doesn't hang
    let html = "<html><head><title>BS TikTok Auto</title></head><body>\
                <p style='font-family:sans-serif;margin:3rem auto;max-width:400px;text-align:center'>\
                <strong>Login successful.</strong><br>You can close this tab and return to BS TikTok Auto.\
                </p></body></html>";
    let ct = Header::from_bytes("Content-Type", "text/html").unwrap();
    let _ = request.respond(Response::from_string(html).with_header(ct));

    // Handle error response from TikTok (e.g., user denied)
    if let Some(err) = params.get("error") {
        let desc = params.get("error_description").cloned().unwrap_or_default();
        let _ = tx.send(AuthResult::Error(format!("TikTok denied: {err} — {desc}")));
        return;
    }

    let code = match params.get("code") {
        Some(c) => c.clone(),
        None => {
            let _ = tx.send(AuthResult::Error("No auth code in callback URL.".into()));
            return;
        }
    };

    let returned_state = params.get("state").cloned().unwrap_or_default();
    if returned_state != expected_state {
        let _ = tx.send(AuthResult::Error(
            "State mismatch — possible CSRF attack. Try again.".into(),
        ));
        return;
    }

    match tiktok_auth::exchange_code(&client_key, &client_secret, &code, &code_verifier) {
        Ok(token) => {
            let _ = tx.send(AuthResult::Token(token));
        }
        Err(e) => {
            let _ = tx.send(AuthResult::Error(e));
        }
    }
}
