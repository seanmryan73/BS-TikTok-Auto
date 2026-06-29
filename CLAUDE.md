# BS-TikTok-Auto — Claude Context

Rust egui pipeline that crops horizontal Fortnite gameplay clips to 9:16 vertical format and uploads them to TikTok via the Content Posting API. Background worker threads handle ffmpeg crop+encode and TikTok chunked upload; the egui UI shows the job queue, auth status, and upload progress.

## Shared reference notes

@c:\_repos\Obsidian\Notes\Claude\Reference\Author-Version-Standards.md
@c:\_repos\Obsidian\Notes\Claude\Reference\Rust-Desktop-Standards.md
@c:\_repos\Obsidian\Notes\Claude\Reference\Windows-Platform-Gotchas.md

## Project context

@c:\_repos\Obsidian\Notes\Claude\Projects\BS-TikTok-Auto Claude Context.md

## Critical constraints

- **egui/eframe pinned at 0.34** — do not upgrade without updating all egui-family crates together.
- **No tokio / no async in egui** — all HTTP and I/O runs on `std::thread::spawn` workers; results return via `mpsc` channels. Do not add tokio or reqwest.
- **Use `ureq` with `rustls` feature** — `ureq = { version = "2", features = ["rustls"] }`. No openssl dependency.
- **`unsafe` NOT required** — this app makes no direct Windows API calls. Add `#![deny(unsafe_code)]`.
- **Credentials via env vars only** — `TIKTOK_CLIENT_KEY` and `TIKTOK_CLIENT_SECRET`. Never hardcode; never write credentials to settings.json or any file.
- **TikTok scopes: `video.upload` + `video.publish`** — the PS test script used `user.comments.read` which is wrong for video posting. Use the video scopes.
- **ffmpeg subprocess** — crop/scale via external ffmpeg.exe. Use `.creation_flags(0x08000000)` (`CREATE_NO_WINDOW`) on all `std::process::Command` spawns.
- **%APPDATA% only** — `%APPDATA%\BSTikTokAuto\` for settings.json, tokens.json, and processed output. Never write to the install directory.
- **5-theme system** — CoralStorm, CandyPop, GlitchMode, ColdSteel, Lucky. Lucky is the default.

## Working rules

- Follow Rust-Desktop-Standards.md unless the project note documents a deliberate exception.
- Prefer minimal, targeted edits.

## After this session

When the session ends or the user says to wrap up, update the project context note:
`c:\_repos\Obsidian\Notes\Claude\Projects\BS-TikTok-Auto Claude Context.md`

Update these sections:
- **Current constraints** — add any new version pins, banned patterns, or architecture rules discovered
- **Fix history** — add bugs fixed with root cause (one line each: date · symptom · cause · fix)
- **Next actions** — replace with the current list
- **Phase status** — mark completed phases ✅
- **frontmatter `version:`** — set to today's date (YYYY.MM.DD)
