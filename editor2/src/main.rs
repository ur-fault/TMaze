use eframe::{
    self,
    egui::{self, plot, Sense},
    epaint::{Pos2, Rect, Rounding, Stroke},
    App, NativeOptions,
};
fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "TMaze Editor",
        NativeOptions {
            ..Default::default()
        },
        Box::new(|_cc| Box::new(EditorApp::new())),
    )
}

struct EditorApp {}

impl EditorApp {
    fn new() -> Self {
        Self {}
    }
}

impl App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
            let response = ui.allocate_response(rect.size(), Sense::drag());
            let painter = ui.painter_at(rect);
            if let Some(pos) = response.hover_pos() {
                painter.line_segment(
                    [Pos2::new(0.0, pos.y), Pos2::new(rect.width(), pos.y)],
                    Stroke::new(1.0, egui::Color32::RED),
                );
                painter.line_segment(
                    [Pos2::new(pos.x, 0.0), Pos2::new(pos.x, rect.height())],
                    Stroke::new(1.0, egui::Color32::RED),
                );
            }
        });
    }
}
