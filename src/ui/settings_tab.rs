use crate::settings::AppSettings;
use crate::theme::ThemeChoice;
use eframe::egui;

/// Returns true if any setting changed (caller should save).
pub fn show(ui: &mut egui::Ui, settings: &mut AppSettings, theme_choice: &mut ThemeChoice) -> bool {
    let mut changed = false;

    ui.add_space(8.0);
    ui.heading("Settings");
    ui.add_space(8.0);

    egui::Grid::new("settings_grid")
        .num_columns(2)
        .spacing([16.0, 8.0])
        .show(ui, |ui| {
            ui.label("Theme");
            let mut theme_idx = ThemeChoice::ALL.iter().position(|t| *t == *theme_choice).unwrap_or(4);
            egui::ComboBox::from_id_salt("settings_theme")
                .selected_text(theme_choice.label())
                .show_index(ui, &mut theme_idx, ThemeChoice::ALL.len(), |i| {
                    ThemeChoice::ALL[i].label()
                });
            let chosen = ThemeChoice::ALL[theme_idx];
            if chosen != *theme_choice {
                *theme_choice = chosen;
                changed = true;
            }
            ui.end_row();

            ui.label("Video Quality");
            let mut quality_idx = crate::settings::app_settings::VideoQuality::ALL
                .iter()
                .position(|q| *q == settings.quality)
                .unwrap_or(0);
            egui::ComboBox::from_id_salt("settings_quality")
                .selected_text(settings.quality.label())
                .show_index(
                    ui,
                    &mut quality_idx,
                    crate::settings::app_settings::VideoQuality::ALL.len(),
                    |i| crate::settings::app_settings::VideoQuality::ALL[i].label(),
                );
            let chosen_q = crate::settings::app_settings::VideoQuality::ALL[quality_idx];
            if chosen_q != settings.quality {
                settings.quality = chosen_q;
                changed = true;
            }
            ui.end_row();

            ui.label("Clip Source Dir");
            ui.label(&settings.clip_output_dir);
            ui.end_row();

            ui.label("");
            if ui.button("Open folder").clicked() {
                let _ = std::process::Command::new("explorer")
                    .arg(&settings.clip_output_dir)
                    .spawn();
            }
            ui.end_row();
        });

    changed
}
