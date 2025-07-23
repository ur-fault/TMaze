use unicode_width::UnicodeWidthStr;

use cmaze::dims::Dims;

use super::{draw_fn::*, *};

pub struct ProgressBar {
    title: String,
    progress: f64,
}

impl ProgressBar {
    pub fn new(title: String) -> Self {
        Self {
            title,
            progress: 0.,
        }
    }

    pub fn update_progress(&mut self, progress: f64) {
        self.progress = progress;
    }

    pub fn update_title(&mut self, title: String) {
        self.title = title;
    }
}

impl Screen for ProgressBar {
    fn draw(&mut self, frame: &mut Frame, theme: &Theme) -> io::Result<()> {
        let progress_size = Dims(self.title.width() as i32 + 2 + 2, 4);
        let pos = center_box_in_screen(progress_size);

        let prg = "█".repeat((self.title.width() as f64 * self.progress) as usize);

        let box_style = theme["ui.progressbar.border"];
        let text_style = theme["ui.progressbar.text"];
        let prg_style = theme["ui.progressbar.progress"];

        draw_box(frame, pos, progress_size, box_style);
        frame.draw(pos + Dims(2, 1), self.title.as_str(), text_style);
        frame.draw(pos + Dims(2, 2), prg, prg_style);

        Ok(())
    }
}

pub fn progressbar_theme_resolver() -> ThemeResolver {
    let mut resolver = ThemeResolver::new();

    resolver
        .link("ui.progressbar.border", "border")
        .link("ui.progressbar.text", "text")
        .link("ui.progressbar.progress", "border");

    resolver
}
