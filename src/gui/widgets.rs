use eframe::egui;
use eframe::egui::{Response, TextBuffer, Ui};

pub struct StatusIndicator {
    status_text: String,
    arena_text: Option<String>,
    color: egui::Color32,
    circle_radius: f32,
}

impl StatusIndicator {
    pub fn connected(arena_text: String) -> Self {
        Self {
            status_text: "Connected".to_string(),
            arena_text: Some(arena_text),
            color: egui::Color32::GREEN,
            circle_radius: 7.5,
        }
    }

    pub fn connecting() -> Self {
        Self {
            status_text: "Connecting".to_string(),
            arena_text: None,
            color: egui::Color32::ORANGE,
            circle_radius: 7.5,
        }
    }

    pub fn disconnected() -> Self {
        Self {
            status_text: "Disconnected".to_string(),
            arena_text: None,
            color: egui::Color32::RED,
            circle_radius: 7.5,
        }
    }
}

impl egui::Widget for StatusIndicator {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.add_space(ui.spacing().item_spacing.x);

            let (rect, _response) = ui.allocate_exact_size(
                egui::vec2(self.circle_radius * 2.0, self.circle_radius * 2.0),
                egui::Sense::hover()
            );
            ui.painter().circle_filled(
                rect.center(),
                self.circle_radius,
                self.color
            );
            ui.label(&self.status_text);
            if self.arena_text.is_some() {
                ui.label(format!("({})", &self.arena_text.unwrap_or_default()));
            }
        })
            .response
    }
}

pub struct TinyTextEdit<'t> {
    label: &'t str,
    text: &'t mut dyn TextBuffer,
    width: Option<f32>,
}

impl<'t> TinyTextEdit<'t> {
    pub fn single_line(label: &'t str, text: &'t mut dyn TextBuffer) -> Self {
        Self {
            label,
            text,
            width: None,
        }
    }
    
    pub fn with_desired_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl<'t> egui::Widget for TinyTextEdit<'t> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(self.text)
                    .desired_width(self.width.unwrap_or_else(|| 135.0))
            );
            ui.label(self.label);
        }).response
    }
}

pub struct PlayerList<'v> {
    players: &'v Vec<String>,
}

impl<'v> PlayerList<'v> {
    pub fn list(players: &'v Vec<String>) -> Self {
        Self {
            players
        }
    }
}

impl<'v> egui::Widget for PlayerList<'v> {
    fn ui(self, ui: &mut Ui) -> Response {
        egui::Frame::new()
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(4.0)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0; // tighter spacing
                    for (i, name) in self.players.iter().enumerate() {
                        if i > 0 {
                            ui.separator();
                        }

                        ui.add_sized(
                            [ui.available_width(), 0.0],
                            egui::Label::new(name).truncate()
                        );
                    }
                });
            }).response
    }
}