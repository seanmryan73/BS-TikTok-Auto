use crate::pipeline::{JobStatus, PipelineJob};
use eframe::egui::{self, Color32};
use std::path::PathBuf;

pub enum QueueAction {
    None,
    AddFiles(Vec<PathBuf>),
    RemoveJob(u64),
}

pub fn show(ui: &mut egui::Ui, jobs: &[PipelineJob], connected: bool) -> QueueAction {
    let mut action = QueueAction::None;

    ui.add_space(8.0);
    ui.heading("Queue");
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        if ui
            .add_enabled(connected, egui::Button::new("Add Clips…"))
            .clicked()
        {
            if let Some(paths) = rfd::FileDialog::new()
                .add_filter("MP4 video", &["mp4"])
                .pick_files()
            {
                action = QueueAction::AddFiles(paths);
            }
        }
        if !connected {
            ui.weak("Connect TikTok in the Auth tab before adding clips.");
        }
    });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    if jobs.is_empty() {
        ui.weak("No clips queued. Click \"Add Clips…\" to crop and upload a video.");
        return action;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for job in jobs {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    let name = job
                        .input_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    ui.label(egui::RichText::new(name).strong());

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if job.status.is_finished() && ui.small_button("Remove").clicked() {
                            action = QueueAction::RemoveJob(job.id);
                        }
                    });
                });

                ui.add_space(4.0);

                match &job.status {
                    JobStatus::Queued => {
                        ui.weak("Queued");
                    }
                    JobStatus::Cropping => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Cropping & encoding…");
                        });
                    }
                    JobStatus::Uploading(p) => {
                        ui.add(egui::ProgressBar::new(*p).text(format!("Uploading {:.0}%", p * 100.0)));
                    }
                    JobStatus::Publishing => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Publishing to TikTok…");
                        });
                    }
                    JobStatus::Done => {
                        ui.colored_label(Color32::from_rgb(0x44, 0xdd, 0x66), "Done — posted as private (SELF_ONLY)");
                    }
                    JobStatus::Failed(e) => {
                        ui.colored_label(Color32::from_rgb(0xff, 0x55, 0x22), format!("Failed: {e}"));
                    }
                }
            });
            ui.add_space(6.0);
        }
    });

    action
}
