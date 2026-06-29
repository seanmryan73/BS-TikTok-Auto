use crate::auth::{self, AuthResult, AuthStatus, TokenData};
use crate::settings::{self, AppSettings, SettingsFile};
use crate::theme::{ThemeChoice, ThemeManager};
use crate::ui;
use eframe::egui::{self, Context};
use std::sync::mpsc;

#[derive(PartialEq, Clone, Copy)]
pub enum Tab {
    Queue,
    Auth,
    Settings,
    About,
}

pub struct App {
    pub settings: SettingsFile,
    pub theme: ThemeManager,
    pub active_tab: Tab,
    pub reset_confirm: bool,

    // Auth state
    pub token: Option<TokenData>,
    pub auth_status: AuthStatus,
    pub auth_rx: Option<mpsc::Receiver<AuthResult>>,
    pub auth_stop_tx: Option<mpsc::SyncSender<()>>,
    pub pending_state: String,

    // Credential input buffers (never persisted)
    pub cred_key: String,
    pub cred_secret: String,

    // PKCE verifier stored until token exchange completes
    pub code_verifier: String,

    // Last auth URL attempted — shown in UI for debugging
    pub last_auth_url: String,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings = settings::load();
        let mut theme = ThemeManager::new();
        theme.choice = settings.app.theme;
        theme.apply(&cc.egui_ctx);

        let token = auth::token_store::load();
        let auth_status = if token.is_some() {
            AuthStatus::Connected
        } else {
            AuthStatus::Disconnected
        };

        Self {
            settings,
            theme,
            active_tab: Tab::Queue,
            reset_confirm: false,
            token,
            auth_status,
            auth_rx: None,
            auth_stop_tx: None,
            pending_state: String::new(),
            cred_key: String::new(),
            cred_secret: String::new(),
            code_verifier: String::new(),
            last_auth_url: String::new(),
        }
    }

    pub fn save_settings(&self) {
        settings::save(&self.settings);
    }

    fn start_oauth(&mut self) {
        let key = match std::env::var("TIKTOK_CLIENT_KEY") {
            Ok(k) => k,
            Err(_) => {
                self.auth_status =
                    AuthStatus::Error("TIKTOK_CLIENT_KEY environment variable not set.".into());
                return;
            }
        };
        let secret = match std::env::var("TIKTOK_CLIENT_SECRET") {
            Ok(s) => s,
            Err(_) => {
                self.auth_status =
                    AuthStatus::Error("TIKTOK_CLIENT_SECRET environment variable not set.".into());
                return;
            }
        };

        let state = auth::tiktok_auth::gen_state();
        self.pending_state = state.clone();

        let (verifier, challenge) = auth::tiktok_auth::gen_pkce_pair();
        self.code_verifier = verifier.clone();

        let url = auth::tiktok_auth::build_auth_url(&key, &state, &challenge);
        self.last_auth_url = url.clone();
        let _ = open::that(&url);

        let (result_tx, result_rx) = mpsc::channel::<AuthResult>();
        let (stop_tx, stop_rx) = mpsc::sync_channel::<()>(1);

        auth::callback_server::start(key, secret, state, verifier, result_tx, stop_rx);

        self.auth_rx = Some(result_rx);
        self.auth_stop_tx = Some(stop_tx);
        self.auth_status = AuthStatus::Connecting;
    }

    fn save_credentials(&mut self) {
        use std::os::windows::process::CommandExt;
        let key = std::mem::take(&mut self.cred_key);
        let secret = std::mem::take(&mut self.cred_secret);
        // Make available in the current process immediately
        std::env::set_var("TIKTOK_CLIENT_KEY", &key);
        std::env::set_var("TIKTOK_CLIENT_SECRET", &secret);
        // Persist to Windows user environment via PowerShell (no inline interpolation —
        // values are passed as env vars to avoid any quoting/injection issues)
        let _ = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "[Environment]::SetEnvironmentVariable('TIKTOK_CLIENT_KEY',$env:_KEY,'User');\
                 [Environment]::SetEnvironmentVariable('TIKTOK_CLIENT_SECRET',$env:_SECRET,'User')",
            ])
            .env("_KEY", &key)
            .env("_SECRET", &secret)
            .creation_flags(0x08000000)
            .spawn();
    }

    fn cancel_oauth(&mut self) {
        if let Some(tx) = self.auth_stop_tx.take() {
            let _ = tx.send(());
        }
        self.auth_rx = None;
        self.auth_status = AuthStatus::Disconnected;
    }

    fn disconnect(&mut self) {
        self.cancel_oauth();
        auth::token_store::clear();
        self.token = None;
        self.auth_status = AuthStatus::Disconnected;
    }

    fn poll_auth(&mut self, ctx: &Context) {
        if let Some(rx) = &self.auth_rx {
            match rx.try_recv() {
                Ok(AuthResult::Token(token)) => {
                    auth::token_store::save(&token);
                    self.token = Some(token);
                    self.auth_status = AuthStatus::Connected;
                    self.auth_rx = None;
                    self.auth_stop_tx = None;
                    ctx.request_repaint();
                }
                Ok(AuthResult::Error(e)) => {
                    self.auth_status = AuthStatus::Error(e);
                    self.auth_rx = None;
                    self.auth_stop_tx = None;
                    ctx.request_repaint();
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // Still waiting — keep the UI refreshing so the spinner animates
                    if self.auth_status == AuthStatus::Connecting {
                        ctx.request_repaint_after(std::time::Duration::from_millis(500));
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Thread exited without sending — treat as cancelled
                    self.auth_rx = None;
                    self.auth_stop_tx = None;
                    if self.auth_status == AuthStatus::Connecting {
                        self.auth_status = AuthStatus::Disconnected;
                    }
                }
            }
        }
    }

    fn show_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.strong("BS TikTok Auto");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Exit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if ui.small_button("?").clicked() {
                    self.active_tab = Tab::About;
                }
                ui.separator();
                let mut chosen = self.theme.choice;
                egui::ComboBox::from_id_salt("theme_selector")
                    .selected_text(self.theme.choice.label())
                    .width(110.0)
                    .show_ui(ui, |ui| {
                        for &t in ThemeChoice::ALL {
                            if ui.add(egui::Button::new(t.label()).selected(t == chosen)).clicked() {
                                chosen = t;
                            }
                        }
                    });
                if chosen != self.theme.choice {
                    self.theme.choice = chosen;
                    self.settings.app.theme = chosen;
                    self.save_settings();
                }
            });
        });
    }

    fn show_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for (tab, label) in [
                (Tab::Queue, "Queue"),
                (Tab::Auth, "Auth"),
                (Tab::Settings, "Settings"),
                (Tab::About, "About"),
            ] {
                let selected = self.active_tab == tab;
                if ui.add(egui::Button::new(label).selected(selected)).clicked() {
                    self.active_tab = tab;
                }
            }
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.theme.apply(&ctx);
        self.poll_auth(&ctx);

        egui::Panel::top("header").show_inside(ui, |ui| {
            self.show_header(ui);
            ui.separator();
            self.show_tabs(ui);
        });

        egui::CentralPanel::default().show_inside(ui, |ui| match self.active_tab {
            Tab::Queue => ui::queue_tab::show(ui),
            Tab::Auth => {
                let action = ui::auth_tab::show(
                    ui,
                    self.token.as_ref(),
                    &self.auth_status,
                    &mut self.cred_key,
                    &mut self.cred_secret,
                    &self.last_auth_url,
                );
                match action {
                    ui::auth_tab::AuthAction::Connect => self.start_oauth(),
                    ui::auth_tab::AuthAction::Cancel => self.cancel_oauth(),
                    ui::auth_tab::AuthAction::Disconnect => self.disconnect(),
                    ui::auth_tab::AuthAction::SaveCredentials => self.save_credentials(),
                    ui::auth_tab::AuthAction::None => {}
                }
            }
            Tab::Settings => {
                let changed =
                    ui::settings_tab::show(ui, &mut self.settings.app, &mut self.theme.choice);
                if changed {
                    self.settings.app.theme = self.theme.choice;
                    self.save_settings();
                }
            }
            Tab::About => {
                let reset = ui::about_tab::show(ui, &mut self.reset_confirm);
                if reset {
                    self.settings.app = AppSettings::default();
                    self.theme.choice = self.settings.app.theme;
                    self.save_settings();
                }
            }
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_settings();
    }
}
