use crate::auth::TokenData;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::time::{Duration, Instant};

const INIT_ENDPOINT: &str = "https://open.tiktokapis.com/v2/post/publish/video/init/";
const STATUS_ENDPOINT: &str = "https://open.tiktokapis.com/v2/post/publish/status/fetch/";
const CHUNK_SIZE: u64 = 10 * 1024 * 1024; // 10 MB
const STATUS_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Serialize)]
struct PostInfo<'a> {
    title: &'a str,
    privacy_level: &'a str,
    disable_duet: bool,
    disable_comment: bool,
    disable_stitch: bool,
}

#[derive(Serialize)]
struct SourceInfo {
    source: &'static str,
    video_size: u64,
    chunk_size: u64,
    total_chunk_count: u64,
}

#[derive(Serialize)]
struct InitRequest<'a> {
    post_info: PostInfo<'a>,
    source_info: SourceInfo,
}

#[derive(Deserialize, Default)]
struct ApiError {
    #[serde(default)]
    code: String,
    #[serde(default)]
    message: String,
}

#[derive(Deserialize)]
struct InitResponseData {
    publish_id: String,
    upload_url: String,
}

#[derive(Deserialize)]
struct InitResponse {
    #[serde(default)]
    data: Option<InitResponseData>,
    #[serde(default)]
    error: ApiError,
}

pub struct InitData {
    pub publish_id: String,
    pub upload_url: String,
    pub video_size: u64,
    pub chunk_size: u64,
    pub total_chunk_count: u64,
}

/// Initializes a Content Posting API upload. Until this app passes TikTok's
/// audit, posts must use `privacy_level: SELF_ONLY` — they are only visible
/// to the connected account, never published publicly.
pub fn init_upload(token: &TokenData, video_path: &Path, title: &str) -> Result<InitData, String> {
    let video_size = std::fs::metadata(video_path)
        .map_err(|e| format!("Cannot read video file: {e}"))?
        .len();

    let chunk_size = CHUNK_SIZE.min(video_size.max(1));
    let total_chunk_count = video_size.div_ceil(chunk_size).max(1);

    let req = InitRequest {
        post_info: PostInfo {
            title,
            privacy_level: "SELF_ONLY",
            disable_duet: false,
            disable_comment: false,
            disable_stitch: false,
        },
        source_info: SourceInfo {
            source: "FILE_UPLOAD",
            video_size,
            chunk_size,
            total_chunk_count,
        },
    };

    let body = serde_json::to_vec(&req).map_err(|e| format!("Serialize init request: {e}"))?;
    let resp = ureq::post(INIT_ENDPOINT)
        .set("Authorization", &format!("Bearer {}", token.access_token))
        .set("Content-Type", "application/json; charset=UTF-8")
        .send_bytes(&body)
        .map_err(|e| format!("Init request failed: {e}"))?;

    let init: InitResponse = serde_json::from_reader(resp.into_reader())
        .map_err(|e| format!("Init response parse failed: {e}"))?;

    let data = init
        .data
        .ok_or_else(|| format!("Init error: {} — {}", init.error.code, init.error.message))?;

    Ok(InitData {
        publish_id: data.publish_id,
        upload_url: data.upload_url,
        video_size,
        chunk_size,
        total_chunk_count,
    })
}

/// Uploads the video to TikTok's signed `upload_url` in chunks, reporting
/// fractional progress (0.0..=1.0) after each chunk completes.
pub fn upload_chunks(
    init: &InitData,
    video_path: &Path,
    mut on_progress: impl FnMut(f32),
) -> Result<(), String> {
    let mut file = std::fs::File::open(video_path).map_err(|e| format!("Cannot open video: {e}"))?;
    let mut buf = vec![0u8; init.chunk_size as usize];

    for i in 0..init.total_chunk_count {
        let start = i * init.chunk_size;
        let end = ((i + 1) * init.chunk_size).min(init.video_size) - 1;
        let len = (end - start + 1) as usize;

        file.seek(SeekFrom::Start(start)).map_err(|e| format!("Seek failed: {e}"))?;
        file.read_exact(&mut buf[..len]).map_err(|e| format!("Read failed: {e}"))?;

        let content_range = format!("bytes {start}-{end}/{}", init.video_size);

        ureq::put(&init.upload_url)
            .set("Content-Type", "video/mp4")
            .set("Content-Range", &content_range)
            .send_bytes(&buf[..len])
            .map_err(|e| format!("Chunk {} upload failed: {e}", i + 1))?;

        on_progress((i + 1) as f32 / init.total_chunk_count as f32);
    }

    Ok(())
}

#[derive(Deserialize, Default)]
struct StatusData {
    #[serde(default)]
    status: String,
    #[serde(default)]
    fail_reason: String,
}

#[derive(Deserialize)]
struct StatusResponse {
    #[serde(default)]
    data: Option<StatusData>,
    #[serde(default)]
    error: ApiError,
}

/// Polls publish status until TikTok finishes processing the upload, or
/// times out after 2 minutes.
pub fn poll_status(token: &TokenData, publish_id: &str) -> Result<(), String> {
    let deadline = Instant::now() + STATUS_TIMEOUT;

    loop {
        let body = serde_json::to_vec(&serde_json::json!({ "publish_id": publish_id }))
            .map_err(|e| format!("Serialize status request: {e}"))?;
        let resp = ureq::post(STATUS_ENDPOINT)
            .set("Authorization", &format!("Bearer {}", token.access_token))
            .set("Content-Type", "application/json; charset=UTF-8")
            .send_bytes(&body)
            .map_err(|e| format!("Status request failed: {e}"))?;

        let status: StatusResponse = serde_json::from_reader(resp.into_reader())
            .map_err(|e| format!("Status response parse failed: {e}"))?;

        let data = status
            .data
            .ok_or_else(|| format!("Status error: {} — {}", status.error.code, status.error.message))?;

        match data.status.as_str() {
            "PUBLISH_COMPLETE" | "SEND_TO_USER_INBOX" => return Ok(()),
            "FAILED" => return Err(format!("Publish failed: {}", data.fail_reason)),
            _ => {
                if Instant::now() >= deadline {
                    return Err("Publish status polling timed out.".into());
                }
                std::thread::sleep(Duration::from_secs(3));
            }
        }
    }
}
