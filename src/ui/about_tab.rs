use eframe::egui;

const APP_VERSION: &str = "2026.06.29a";

/// Returns true if Reset to Defaults was confirmed (caller should apply reset).
pub fn show(ui: &mut egui::Ui, reset_confirm: &mut bool) -> bool {
    let mut do_reset = false;

    ui.add_space(8.0);
    ui.heading("BS TikTok Auto");
    ui.label(format!("v{}", APP_VERSION));
    ui.separator();

    ui.label("Crops Fortnite clips to 9:16 vertical and uploads to TikTok.");
    ui.label("Sean Ryan  ·  BagPipes  ·  seanmryan@gmail.com");
    ui.separator();

    ui.strong("Hotkeys");
    egui::Grid::new("about_hotkeys").show(ui, |ui| {
        ui.label("—");
        ui.label("No hotkeys yet");
        ui.end_row();
    });
    ui.separator();

    ui.strong("Settings");
    let path = crate::settings::settings_path();
    ui.label(path.display().to_string());
    if ui.button("Open settings folder").clicked() {
        if let Some(parent) = path.parent() {
            let _ = std::process::Command::new("explorer").arg(parent).spawn();
        }
    }
    ui.separator();

    if *reset_confirm {
        ui.colored_label(
            egui::Color32::from_rgb(0xff, 0x44, 0x22),
            "This will reset all settings to defaults. Are you sure?",
        );
        ui.horizontal(|ui| {
            if ui.button("Cancel").clicked() {
                *reset_confirm = false;
            }
            if ui.button("Confirm Reset").clicked() {
                *reset_confirm = false;
                do_reset = true;
            }
        });
    } else if ui.button("Reset to Defaults").clicked() {
        *reset_confirm = true;
    }

    do_reset
}
