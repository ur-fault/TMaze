use clap::{Parser, Subcommand};
use tmaze::ui::{menu, Renderer};

/// Program for running UI samples from tmaze
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    subcommand: Examples,
}

#[derive(Subcommand, Debug)]
enum Examples {
    Menu
}

fn main() {
    let args = Args::parse();

    let mut renderer = Renderer::default();

    match args.subcommand {
        Examples::Menu => {
            menu::menu(&mut renderer, menu::ContentStyle::default(), menu::ContentStyle::default(), "Menu", &["Option 1", "Option 2", "Option 3"], Some(0), true).unwrap();
        }
    }
}