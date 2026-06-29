use crate::auth::{AuthStatus, TokenData};
use eframe::egui::{self, Color32};

pub enum AuthAction {
    None,
    Connect,
    Cancel,
    Disconnect,
    SaveCredentials,
}

pub fn show(
    ui: &mut egui::Ui,
    token: Option<&TokenData>,
    status: &AuthStatus,
    cred_key: &mut String,
    cred_secret: &mut String,
    last_auth_url: &str,
) -> AuthAction {
    let mut action = AuthAction::None;

    ui.add_space(8.0);
    ui.heading("TikTok Auth");
    ui.add_space(8.0);

    // ── Credentials panel ───────────────────────────────────────────────────
    let key_set = std::env::var("TIKTOK_CLIENT_KEY").is_ok();
    let secret_set = std::env::var("TIKTOK_CLIENT_SECRET").is_ok();
    let creds_ok = key_set && secret_set;

    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label(egui::RichText::new("Developer Credentials").strong());
        ui.add_space(4.0);

        if creds_ok {
            ui.colored_label(Color32::from_rgb(0x44, 0xdd, 0x66), "Credentials configured.");
        } else {
            ui.colored_label(Color32::from_rgb(0xff, 0xaa, 0x22), "Credentials not set.");
            ui.weak("Get your keys from developer.tiktok.com → your app → Keys & tokens.");
        }

        ui.add_space(6.0);

        egui::Grid::new("cred_grid")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                ui.label("Client Key");
                ui.add(
                    egui::TextEdit::singleline(cred_key)
                        .hint_text(if key_set { "already set — paste new to replace" } else { "paste Client Key here" })
                        .desired_width(240.0),
                );
                ui.end_row();

                ui.label("Client Secret");
                ui.add(
                    egui::TextEdit::singleline(cred_secret)
                        .password(true)
                        .hint_text(if secret_set { "already set — paste new to replace" } else { "paste Client Secret here" })
                        .desired_width(240.0),
                );
                ui.end_row();
            });

        ui.add_space(4.0);
        let can_save = !cred_key.is_empty() && !cred_secret.is_empty();
        if ui
            .add_enabled(can_save, egui::Button::new("Save to Windows Environment"))
            .clicked()
        {
            action = AuthAction::SaveCredentials;
        }
    });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    // ── Connection status ────────────────────────────────────────────────────
    match status {
        AuthStatus::Disconnected => {
            ui.label("Not connected to TikTok.");
            ui.add_space(8.0);
            if ui
                .add_enabled(creds_ok, egui::Button::new("Connect TikTok"))
                .clicked()
            {
                action = AuthAction::Connect;
            }
            if !creds_ok {
                ui.add_space(2.0);
                ui.weak("Save credentials above before connecting.");
            }
        }

        AuthStatus::Connecting => {
            ui.label("Waiting for browser login…");
            ui.weak("Complete the TikTok login in your browser. Listening on localhost:8080.");
            ui.add_space(8.0);
            if ui.button("Cancel").clicked() {
                action = AuthAction::Cancel;
            }
        }

        AuthStatus::Connected => {
            ui.colored_label(Color32::from_rgb(0x44, 0xdd, 0x66), "Connected");
            ui.add_space(4.0);

            if let Some(t) = token {
                let secs = t.seconds_until_expiry();
                let expiry_text = if secs > 7200 {
                    format!("Access token expires in {} h", secs / 3600)
                } else if secs > 60 {
                    format!("Access token expires in {} min", secs / 60)
                } else if secs > 0 {
                    format!("Access token expires in {} s", secs)
                } else {
                    "Access token expired — reconnect.".into()
                };

                if secs <= 0 {
                    ui.colored_label(Color32::from_rgb(0xff, 0x55, 0x22), expiry_text);
                } else {
                    ui.label(expiry_text);
                }

                if !t.open_id.is_empty() {
                    ui.add_space(2.0);
                    ui.weak(format!("open_id: {}", t.open_id));
                }
            }

            ui.add_space(8.0);
            if ui.button("Disconnect").clicked() {
                action = AuthAction::Disconnect;
            }
        }

        AuthStatus::Error(e) => {
            ui.colored_label(Color32::from_rgb(0xff, 0x55, 0x22), format!("Error: {e}"));
            ui.add_space(8.0);
            if ui.button("Try Again").clicked() {
                action = AuthAction::Connect;
            }
        }
    }

    // Debug: show last auth URL when connecting or after an error
    if !last_auth_url.is_empty()
        && matches!(status, AuthStatus::Connecting | AuthStatus::Error(_))
    {
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(4.0);
        ui.weak("Last auth URL (for debugging):");
        egui::ScrollArea::horizontal().id_salt("auth_url_scroll").show(ui, |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut last_auth_url.to_owned())
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace),
            );
        });
    }

    action
}
