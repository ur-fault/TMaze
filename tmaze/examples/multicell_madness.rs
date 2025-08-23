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

    let scheme = TerminalColorScheme::named("catppuccin_mocha");
    let mut buf = GBuffer::new(Dims(19, 8), &scheme);

    buf.mut_view().border(style).inside(|f| {
        f.fill(CellContent::styled(
            ' ',
            Style::bg(Color::Named(NamedColor::DarkYellow)),
        ))
        .fill(CellContent::styled('あ', red))
        .centered(Dims(6, 2), |f| {
            f.fill(CellContent::styled(
                '$',
                Style::bg(Color::Named(NamedColor::Blue)),
            ));
        });
    });

    buf.write(&mut stdout()).unwrap();
}
