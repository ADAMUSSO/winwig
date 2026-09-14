use chrono::{Datelike, Local, Timelike};
use eframe::egui;
use egui::epaint::Shadow;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const APP_KEY: &str = "time_widget_settings_v3";
const WINDOW_WIDTH: f32 = 360.0;
const WINDOW_HEIGHT: f32 = 190.0;
const SETTINGS_HEIGHT: f32 = 330.0;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
struct Settings {
    locked: bool,
    click_through: bool,
    show_seconds: bool,
    show_date: bool,
    background_alpha: u8,
    font_size: f32,
    accent: [u8; 3],
    window_level: WindowLevelSetting,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum WindowLevelSetting {
    Normal,
    AlwaysOnTop,
    AlwaysOnBottom,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            locked: false,
            click_through: false,
            show_seconds: false,
            show_date: true,
            background_alpha: 190,
            font_size: 64.0,
            accent: [120, 190, 255],
            window_level: WindowLevelSetting::Normal,
        }
    }
}

impl WindowLevelSetting {
    fn label(self) -> &'static str {
        match self {
            Self::Normal => "Normálne",
            Self::AlwaysOnTop => "Vždy navrchu",
            Self::AlwaysOnBottom => "Vždy dole",
        }
    }

    fn egui_level(self) -> egui::WindowLevel {
        match self {
            Self::Normal => egui::WindowLevel::Normal,
            Self::AlwaysOnTop => egui::WindowLevel::AlwaysOnTop,
            Self::AlwaysOnBottom => egui::WindowLevel::AlwaysOnBottom,
        }
    }
}

struct TimeWidget {
    settings: Settings,
    settings_open: bool,
    current_height: f32,
}

impl TimeWidget {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings: Settings = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, APP_KEY))
            .unwrap_or_default();
        cc.egui_ctx
            .send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                settings.window_level.egui_level(),
            ));
        Self {
            settings,
            settings_open: false,
            current_height: WINDOW_HEIGHT,
        }
    }

    fn update_window(&mut self, ctx: &egui::Context) {
        let desired_height = if self.settings_open {
            SETTINGS_HEIGHT
        } else {
            WINDOW_HEIGHT
        };
        if (desired_height - self.current_height).abs() > 0.5 {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                WINDOW_WIDTH,
                desired_height,
            )));
            self.current_height = desired_height;
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(
            self.settings.locked && self.settings.click_through && !self.settings_open,
        ));
    }

    fn draw_header(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            let drag = ui.add(
                egui::Label::new(
                    egui::RichText::new(if self.settings.locked {
                        "🔒"
                    } else {
                        "⠿  ČAS"
                    })
                    .color(egui::Color32::from_white_alpha(170))
                    .size(13.0),
                )
                .sense(egui::Sense::drag()),
            );
            if drag.dragged() && !self.settings.locked {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("✕").on_hover_text("Zavrieť").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                let lock_icon = if self.settings.locked { "🔓" } else { "🔒" };
                if ui
                    .small_button(lock_icon)
                    .on_hover_text("Zamknúť / odomknúť")
                    .clicked()
                {
                    self.settings.locked = !self.settings.locked;
                    if !self.settings.locked {
                        self.settings.click_through = false;
                    }
                }
                let icon = if self.settings_open { "⌃" } else { "⚙" };
                if ui.small_button(icon).on_hover_text("Nastavenia").clicked() {
                    self.settings_open = !self.settings_open;
                }
            });
        });
    }

    fn draw_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.separator();
        ui.add_space(8.0);
        ui.heading("Nastavenia");
        ui.add_space(4.0);
        ui.checkbox(&mut self.settings.show_seconds, "Zobraziť sekundy");
        ui.checkbox(&mut self.settings.show_date, "Zobraziť dátum");
        ui.add_enabled_ui(self.settings.locked, |ui| {
            ui.checkbox(&mut self.settings.click_through, "Preklikávanie cez widget");
        });
        if !self.settings.locked {
            ui.label(
                egui::RichText::new("Preklikávanie je dostupné iba pri zamknutom widgete.")
                    .small()
                    .weak(),
            );
        }
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label("Alfa pozadia");
            ui.add(egui::Slider::new(
                &mut self.settings.background_alpha,
                30..=255,
            ));
        });
        ui.horizontal(|ui| {
            ui.label("Veľkosť textu");
            ui.add(egui::Slider::new(&mut self.settings.font_size, 32.0..=100.0).suffix(" px"));
        });
        ui.horizontal(|ui| {
            ui.label("Farba dátumu");
            ui.color_edit_button_srgb(&mut self.settings.accent);
        });
        ui.horizontal(|ui| {
            ui.label("Vrstva okna");
            egui::ComboBox::from_id_salt("window-level")
                .selected_text(self.settings.window_level.label())
                .show_ui(ui, |ui| {
                    for level in [
                        WindowLevelSetting::Normal,
                        WindowLevelSetting::AlwaysOnTop,
                        WindowLevelSetting::AlwaysOnBottom,
                    ] {
                        if ui
                            .selectable_value(&mut self.settings.window_level, level, level.label())
                            .clicked()
                        {
                            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                                level.egui_level(),
                            ));
                        }
                    }
                });
        });
        if self.settings.window_level == WindowLevelSetting::AlwaysOnBottom {
            ui.label(
                egui::RichText::new("Widget môže byť prekrytý inými oknami.")
                    .small()
                    .color(egui::Color32::YELLOW),
            );
        }
    }
}

impl eframe::App for TimeWidget {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_KEY, &self.settings);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_window(ctx);
        let now = Local::now();
        let time = if self.settings.show_seconds {
            format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second())
        } else {
            format!("{:02}:{:02}", now.hour(), now.minute())
        };
        let next_tick = if self.settings.show_seconds {
            Duration::from_secs(1)
        } else {
            Duration::from_secs((60 - now.second()).max(1) as u64)
        };
        ctx.request_repaint_after(next_tick);
        let accent = egui::Color32::from_rgb(
            self.settings.accent[0],
            self.settings.accent[1],
            self.settings.accent[2],
        );
        let panel = egui::Frame::none()
            .fill(egui::Color32::from_black_alpha(
                self.settings.background_alpha,
            ))
            .rounding(18.0)
            .shadow(Shadow {
                offset: egui::vec2(0.0, 6.0),
                blur: 20.0,
                spread: 0.0,
                color: egui::Color32::from_black_alpha(100),
            })
            .inner_margin(egui::Margin::symmetric(20.0, 12.0));
        egui::CentralPanel::default().frame(panel).show(ctx, |ui| {
            self.draw_header(ui, ctx);
            ui.add_space(4.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(time)
                        .size(self.settings.font_size)
                        .strong()
                        .color(egui::Color32::WHITE),
                );
                if self.settings.show_date {
                    ui.label(
                        egui::RichText::new(format!(
                            "{:02}.{:02}.{}",
                            now.day(),
                            now.month(),
                            now.year()
                        ))
                        .size((self.settings.font_size * 0.25).max(11.0))
                        .color(accent),
                    );
                }
            });
            if self.settings_open {
                self.draw_settings(ui, ctx);
            }
        });
    }
}

fn primary_monitor_center() -> egui::Pos2 {
    display_info::DisplayInfo::all()
        .ok()
        .and_then(|displays| displays.into_iter().find(|display| display.is_primary))
        .map(|display| {
            egui::pos2(
                display.x as f32 + (display.width as f32 - WINDOW_WIDTH) / 2.0,
                display.y as f32 + (display.height as f32 - WINDOW_HEIGHT) / 2.0,
            )
        })
        .unwrap_or_else(|| egui::pos2(300.0, 200.0))
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(false)
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_position(primary_monitor_center()),
        ..Default::default()
    };
    eframe::run_native(
        "Desktop Time Widget",
        options,
        Box::new(|cc| Ok(Box::new(TimeWidget::new(cc)))),
    )
}
