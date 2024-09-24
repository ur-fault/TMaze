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
    fn draw(&self, frame: &mut Frame, theme: &Theme) -> io::Result<()> {
        let progress_size = Dims(self.title.width() as i32 + 2 + 2, 4);
        let pos = center_box_in_screen(progress_size);

        let prg = "█".repeat((self.title.width() as f64 * self.progress) as usize);

        let box_style = theme.get("ui_progressbar_border");
        let text_style = theme.get("ui_progressbar_text");

        draw_box(frame, pos, progress_size, box_style);
        frame.draw_styled(pos + Dims(2, 1), self.title.as_str(), text_style);
        frame.draw(pos + Dims(2, 2), prg);

        Ok(())
    }
}

pub fn progressbar_theme_resolver() -> ThemeResolver {
    let mut resolver = ThemeResolver::new();

    resolver
        .link("ui_progressbar_border", "border")
        .link("ui_progressbar_text", "text");

    resolver
}
