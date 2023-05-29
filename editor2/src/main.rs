use eframe::{self, egui::{self, plot}, NativeOptions, App};
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
            
        });
    }
}
