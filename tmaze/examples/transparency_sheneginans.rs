use std::{io::stdout, rc::Rc};

use cmaze::dims::Dims;
use tmaze::{
    renderer::{CellContent, GBuffer, RenderMode},
    settings::theme::{Color, NamedColor, Style, TerminalColorScheme},
    ui::Rect,
};

fn main() {
    let scheme = Rc::new(TerminalColorScheme::named("catppuccin_mocha"));

    {
        let mut buf = GBuffer::new(Dims(12, 3), &scheme);
        let colors = [
            NamedColor::DarkRed,
            NamedColor::DarkGreen,
            NamedColor::DarkBlue,
        ];

        buf.mut_view()
            .fill(CellContent::styled('x', Style::default()));

        for x in 3..9 {
            for y in 1..3 {
                buf.mut_view().draw(
                    Dims(x, y),
                    ' ',
                    Style {
                        bg: Some(Color::Named(colors[(x % 3) as usize])),
                        alpha: 255,
                        ..Default::default()
                    },
                );
            }
        }

        buf.write(&mut stdout(), RenderMode::RGB).unwrap();
    }

    {
        let mut buf = GBuffer::new(Dims(64, 32), &scheme);

        for i in 0..32 {
            buf.mut_view().fill_rect(
                Rect::sized_at(Dims(0, i), Dims(64, 1)),
                CellContent::styled(
                    '$',
                    Style {
                        bg: Some(Color::Named(NamedColor::Black)),
                        fg: Some(Color::Named(NamedColor::White)),
                        alpha: (256 / 32 * i) as u8,
                        ..Default::default()
                    },
                ),
            );
        }

        for i in 0..64 {
            buf.mut_view().fill_rect(
                Rect::sized_at(Dims(i, 0), Dims(1, 32)),
                CellContent::styled(
                    ' ',
                    Style {
                        bg: Some(Color::Named(NamedColor::Red)),
                        alpha: (256 / 64 * i) as u8,
                        ..Default::default()
                    },
                ),
            );
        }

        buf.write(&mut stdout(), RenderMode::RGB).unwrap();
    }

    {
        let red = Style {
            fg: Some(Color::Named(NamedColor::Black)),
            bg: Some(Color::Named(NamedColor::Red)),
            ..Default::default()
        };

        let f = |buf: &mut GBuffer, alpha| {
            buf.mut_view().inside(|f| {
                f.fill(CellContent::styled('あ', red))
                    .centered(Dims(6, 2), |f| {
                        f.fill(CellContent::styled(
                            '$',
                            Style {
                                bg: Some(Color::Named(NamedColor::Blue)),
                                alpha,
                                ..Style::default()
                            },
                        ));
                    });
            });
            buf.write(&mut stdout(), RenderMode::RGB).unwrap();
        };

        let mut buf = GBuffer::new(Dims(14, 6), &scheme);

        f(&mut buf, 255);

        buf.mut_view().clear();
        f(&mut buf, 128);

        buf.mut_view().clear();
        f(&mut buf, 0);
    }
}
