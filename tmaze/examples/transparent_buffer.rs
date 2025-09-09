use std::{mem, rc::Rc, sync::Arc};

use cmaze::{
    algorithms::{
        region_generator::RndKruskals, region_splitter::DefaultRegionSplitter, GeneratorRegistry,
        MazeSpec, MazeSpecType, SplitterRegistry,
    },
    dims::{Dims, Dims3D},
    game::{GameProperities, RunningGame},
};
use tmaze::{
    app::{app::init_theme_resolver, game::MazeBoard},
    renderer::{draw::Align, GBuffer, RenderMode},
    settings::theme::{Color, NamedColor, Style, TerminalColorScheme, ThemeDefinition},
};

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

fn grad(hash: i32, x: f32, y: f32) -> f32 {
    match hash & 3 {
        0 => x + y,
        1 => -x + y,
        2 => x - y,
        _ => -x - y,
    }
}

/// Simple 2D Perlin noise at (x, y). Returns roughly in [-1, 1].
pub fn perlin(x: f32, y: f32) -> f32 {
    // Permutation table (classic Perlin uses 256 repeated)
    const P: [i32; 512] = {
        let base: [i32; 256] = [
            151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103,
            30, 69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197,
            62, 94, 252, 219, 203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20,
            125, 136, 171, 168, 68, 175, 74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83,
            111, 229, 122, 60, 211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54,
            65, 25, 63, 161, 1, 216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169, 200, 196, 135,
            130, 116, 188, 159, 86, 164, 100, 109, 198, 173, 186, 3, 64, 52, 217, 226, 250, 124,
            123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206, 59, 227, 47, 16, 58, 17,
            182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44, 154, 163, 70, 221, 153,
            101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232, 178,
            185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162,
            241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157, 184,
            84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67, 29,
            24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
        ];
        let mut arr = [0; 512];
        let mut i = 0;
        while i < 256 {
            arr[i] = base[i];
            arr[i + 256] = base[i];
            i += 1;
        }
        arr
    };

    let xi = x.floor() as i32 & 255;
    let yi = y.floor() as i32 & 255;

    let xf = x - x.floor();
    let yf = y - y.floor();

    let u = fade(xf);
    let v = fade(yf);

    let aa = P[(P[xi as usize] + yi) as usize];
    let ab = P[(P[xi as usize] + yi + 1) as usize];
    let ba = P[(P[xi as usize + 1] + yi) as usize];
    let bb = P[(P[xi as usize + 1] + yi + 1) as usize];

    let x1 = lerp(grad(aa, xf, yf), grad(ba, xf - 1.0, yf), u);
    let x2 = lerp(grad(ab, xf, yf - 1.0), grad(bb, xf - 1.0, yf - 1.0), u);

    lerp(x1, x2, v)
}

fn main() {
    let scheme = Rc::new(TerminalColorScheme::named("catppuccin_mocha").unwrap());
    let mut buf = GBuffer::new(Dims(64, 32), &scheme);

    for y in 0..32 {
        for x in 0..64 {
            let n = perlin(x as f32 / 8.0, y as f32 * 2.0 / 8.0);
            let alpha = (((n + 1.0) / 2.0) * 255.0) as u8;
            buf.mut_view().draw(
                Dims(x, y),
                ' ',
                Style {
                    bg: Some(Color::Named(NamedColor::Red)),
                    alpha,
                    ..Default::default()
                },
            );
        }
    }

    let game = RunningGame::prepare(
        GameProperities {
            maze_spec: MazeSpec {
                inner_spec: MazeSpecType::Simple {
                    size: Some(Dims3D(10, 5, 1)),
                    start: None,
                    end: None,
                    mask: None,
                    splitter: None,
                    generator: None,
                },
                seed: None,
                maze_type: None,
            },
        },
        &GeneratorRegistry::with_default(Arc::new(RndKruskals), "rnd_kruskals"),
        &SplitterRegistry::with_default(Arc::new(DefaultRegionSplitter), "default"),
    )
    .unwrap()
    .handle
    .join()
    .unwrap()
    .unwrap();

    let board: Vec<GBuffer> = unsafe {
        mem::transmute(MazeBoard::new(
            &game,
            &init_theme_resolver().resolve(&ThemeDefinition::parse_default()),
            scheme.clone(),
        ))
    };

    let board_view = board[0].view();

    buf.mut_view().centered(board_view.size(), |f| {
        f.alpha(200, |v| v.draw_aligned(Align::Center, board_view, ()));
    });

    buf.write(&mut std::io::stdout(), RenderMode::RGB).unwrap();
}
