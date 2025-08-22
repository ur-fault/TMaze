use std::io::stdout;

use cmaze::dims::Dims;
use tmaze::{
    renderer::{CellContent, GBuffer},
    settings::theme::{Color, NamedColor, Style, TerminalColorScheme},
};

fn main() {
    let style = Style::default();
    let red = Style {
        fg: Some(Color::Named(NamedColor::Black)),
        bg: Some(Color::Named(NamedColor::Red)),
        ..Default::default()
    };

    let mut buf = GBuffer::new(Dims(19, 8));
    let scheme = TerminalColorScheme::named("catppuccin_mocha");

    buf.mut_view().border(style, &scheme).inside(|f| {
        f.fill(
            CellContent::styled(' ', Style::bg(Color::Named(NamedColor::DarkYellow))),
            &scheme,
        )
        .fill(CellContent::styled('あ', red), &scheme)
        .centered(Dims(6, 2), |f| {
            f.fill(
                CellContent::styled('$', Style::bg(Color::Named(NamedColor::Blue))),
                &scheme,
            );
        });
    });
    buf.write(&mut stdout()).unwrap();
}
