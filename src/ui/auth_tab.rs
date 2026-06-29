use eframe::egui;

pub fn show(ui: &mut egui::Ui) {
    ui.add_space(20.0);
    ui.vertical_centered(|ui| {
        ui.heading("TikTok Auth");
        ui.add_space(8.0);
        ui.label("OAuth 2.0 login coming in Phase 2.");
        ui.add_space(4.0);
        ui.label("Connect your TikTok account to enable video uploads.");
    });
}
