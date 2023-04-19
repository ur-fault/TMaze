use crossterm::style::{self, ContentStyle};
use tmaze::{
    renderer::Renderer,
    ui::{input, wait_for_key},
};

fn main() {
    let mut renderer = Renderer::new().unwrap();
    input(
        &mut renderer,
        ContentStyle {
            foreground_color: Some(style::Color::Red),
            ..Default::default()
        },
        Default::default(),
        "Title",
        None,
        // Option::<String>::None,
        Some("Watermark"),
    )
    .map(|r| println!("Result: {:?}", r))
    .unwrap();

    wait_for_key().unwrap();
}
