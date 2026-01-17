use eframe::egui;
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
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            let (rect, _response) = ui.allocate_exact_size(
                egui::vec2(self.circle_radius * 2.0, self.circle_radius * 2.0),
                egui::Sense::hover()
            );
            ui.painter().circle_filled(
                rect.center(),
                self.circle_radius,
                self.color
            );
            ui.add_space(-5.0);
            ui.label(&self.status_text);
            if self.arena_text.is_some() {
                ui.add_space(-5.0);
                ui.label(format!("({})", &self.arena_text.unwrap_or_default()));
            }
        })
            .response
    }
}