pub mod crop;
pub mod upload;

use crate::auth::TokenData;
use std::path::PathBuf;
use std::sync::mpsc;

#[derive(Clone, Debug, PartialEq)]
pub enum JobStatus {
    Queued,
    Cropping,
    Uploading(f32),
    Publishing,
    Done,
    Failed(String),
}

impl JobStatus {
    pub fn is_finished(&self) -> bool {
        matches!(self, JobStatus::Done | JobStatus::Failed(_))
    }
}

pub struct PipelineJob {
    pub id: u64,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub status: JobStatus,
}

pub enum JobUpdate {
    Status(u64, JobStatus),
}

/// Spawns a worker thread that crops/encodes the clip, then uploads and
/// publishes it to TikTok. Progress and terminal state are reported back
/// to the UI thread via `tx`.
pub fn spawn_job(
    id: u64,
    input_path: PathBuf,
    output_path: PathBuf,
    cq: u32,
    token: TokenData,
    tx: mpsc::Sender<JobUpdate>,
) {
    std::thread::spawn(move || run_job(id, input_path, output_path, cq, token, tx));
}

fn run_job(
    id: u64,
    input_path: PathBuf,
    output_path: PathBuf,
    cq: u32,
    token: TokenData,
    tx: mpsc::Sender<JobUpdate>,
) {
    let _ = tx.send(JobUpdate::Status(id, JobStatus::Cropping));

    if let Err(e) = crop::crop_and_encode(&input_path, &output_path, cq) {
        let _ = tx.send(JobUpdate::Status(id, JobStatus::Failed(e)));
        return;
    }

    let _ = tx.send(JobUpdate::Status(id, JobStatus::Uploading(0.0)));

    let title = input_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "BS TikTok Auto clip".into());

    let init = match upload::init_upload(&token, &output_path, &title) {
        Ok(init) => init,
        Err(e) => {
            let _ = tx.send(JobUpdate::Status(id, JobStatus::Failed(e)));
            return;
        }
    };

    let progress_tx = tx.clone();
    let upload_result = upload::upload_chunks(&init, &output_path, |p| {
        let _ = progress_tx.send(JobUpdate::Status(id, JobStatus::Uploading(p)));
    });

    if let Err(e) = upload_result {
        let _ = tx.send(JobUpdate::Status(id, JobStatus::Failed(e)));
        return;
    }

    let _ = tx.send(JobUpdate::Status(id, JobStatus::Publishing));

    match upload::poll_status(&token, &init.publish_id) {
        Ok(()) => {
            let _ = tx.send(JobUpdate::Status(id, JobStatus::Done));
        }
        Err(e) => {
            let _ = tx.send(JobUpdate::Status(id, JobStatus::Failed(e)));
        }
    }
}
