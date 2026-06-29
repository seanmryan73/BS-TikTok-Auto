use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Clone, Copy)]
enum Encoder {
    Nvenc,
    Qsv,
    Amf,
    Libx264,
}

const ENCODER_CHAIN: [Encoder; 4] =
    [Encoder::Nvenc, Encoder::Qsv, Encoder::Amf, Encoder::Libx264];

impl Encoder {
    fn args(self, cq: u32) -> Vec<String> {
        match self {
            Encoder::Nvenc => vec![
                "-c:v".into(), "h264_nvenc".into(),
                "-preset".into(), "p4".into(),
                "-cq".into(), cq.to_string(),
            ],
            Encoder::Qsv => vec![
                "-c:v".into(), "h264_qsv".into(),
                "-global_quality".into(), cq.to_string(),
            ],
            Encoder::Amf => vec![
                "-c:v".into(), "h264_amf".into(),
                "-quality".into(), "balanced".into(),
                "-qp_i".into(), cq.to_string(),
                "-qp_p".into(), cq.to_string(),
            ],
            Encoder::Libx264 => vec![
                "-c:v".into(), "libx264".into(),
                "-preset".into(), "medium".into(),
                "-crf".into(), cq.to_string(),
            ],
        }
    }
}

/// Crops the center 9:16 strip from a 16:9 clip, scales to 1080x1920, and
/// encodes audio as AAC. Tries NVENC -> QSV -> AMF -> libx264 in order,
/// falling back to the next encoder if the current one fails to run.
pub fn crop_and_encode(input: &Path, output: &Path, cq: u32) -> Result<(), String> {
    let ffmpeg = ffmpeg_path();
    let mut last_err = String::new();

    for encoder in ENCODER_CHAIN {
        let _ = std::fs::remove_file(output);
        match run_ffmpeg(&ffmpeg, input, output, encoder, cq) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = e,
        }
    }

    let _ = std::fs::remove_file(output);
    Err(format!(
        "All encoders failed (NVENC, QSV, AMF, libx264). Last error: {last_err}"
    ))
}

fn run_ffmpeg(
    ffmpeg: &Path,
    input: &Path,
    output: &Path,
    encoder: Encoder,
    cq: u32,
) -> Result<(), String> {
    let status = Command::new(ffmpeg)
        .arg("-y")
        .arg("-i").arg(input)
        .arg("-vf").arg("crop=ih*9/16:ih:(iw-ih*9/16)/2:0,scale=1080:1920")
        .args(encoder.args(cq))
        .arg("-c:a").arg("aac")
        .arg("-b:a").arg("192k")
        .arg("-movflags").arg("+faststart")
        .arg(output)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|e| format!("Failed to launch ffmpeg: {e}"))?;

    if status.success() && output.exists() {
        Ok(())
    } else {
        Err(format!("ffmpeg exited with {status}"))
    }
}

fn ffmpeg_path() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_default();
    let local = PathBuf::from(appdata).join("BSTikTokAuto").join("ffmpeg.exe");
    if local.exists() {
        local
    } else {
        PathBuf::from("ffmpeg.exe")
    }
}
