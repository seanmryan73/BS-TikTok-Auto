use crate::settings::{self, AppSettings, SettingsFile};
use crate::theme::{ThemeChoice, ThemeManager};
use crate::ui;
use eframe::egui::{self, Context};

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
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings = settings::load();
        let mut theme = ThemeManager::new();
        theme.choice = settings.app.theme;
        theme.apply(&cc.egui_ctx);
        Self {
            settings,
            theme,
            active_tab: Tab::Queue,
            reset_confirm: false,
        }
    }

    pub fn save_settings(&self) {
        settings::save(&self.settings);
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
                let mut theme_idx = ThemeChoice::ALL
                    .iter()
                    .position(|t| *t == self.theme.choice)
                    .unwrap_or(4);
                egui::ComboBox::from_id_salt("theme_selector")
                    .selected_text(self.theme.choice.label())
                    .show_index(ui, &mut theme_idx, ThemeChoice::ALL.len(), |i| {
                        ThemeChoice::ALL[i].label()
                    });
                let chosen = ThemeChoice::ALL[theme_idx];
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

        egui::Panel::top("header").show_inside(ui, |ui| {
            self.show_header(ui);
            ui.separator();
            self.show_tabs(ui);
        });

        egui::CentralPanel::default().show_inside(ui, |ui| match self.active_tab {
            Tab::Queue => ui::queue_tab::show(ui),
            Tab::Auth => ui::auth_tab::show(ui),
            Tab::Settings => {
                let changed = ui::settings_tab::show(ui, &mut self.settings.app, &mut self.theme.choice);
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
