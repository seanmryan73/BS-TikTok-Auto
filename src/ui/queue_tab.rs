use eframe::egui;

pub fn show(ui: &mut egui::Ui) {
    ui.add_space(20.0);
    ui.vertical_centered(|ui| {
        ui.heading("Queue");
        ui.add_space(8.0);
        ui.label("Clip queue coming in Phase 3.");
        ui.add_space(4.0);
        ui.label("Drop MP4 files here to crop and upload them to TikTok.");
    });
}
