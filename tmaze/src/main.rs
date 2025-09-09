use tmaze::{
    app::{app::init_theme_resolver, game::MainMenu, Activity, App, GameError},
    helpers::constants::paths::{save_data_path, settings_path},
    settings::Settings,
};

#[cfg(feature = "updates")]
use tmaze::updates;

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[clap(version, author, about, name = "tmaze")]
struct Args {
    #[clap(long, action, help = "Reset config to default and quit")]
    reset_config: bool,
    #[clap(short, long, action, help = "Show config path and quit")]
    show_config_path: bool,
    #[clap(long, help = "Show config in debug format and quit")]
    debug_config: bool,
    #[clap(short, long, action, help = "Delete all saved data and quit")]
    delete_data: bool,
    #[clap(
        short,
        long,
        action,
        help = "Run in read-only mode, no data will be saved"
    )]
    read_only: bool,
    #[clap(
        long = "print-styles",
        value_name = "MODE",
        help = "Print available theme options and quit"
    )]
    print_theme_options: Option<Option<StylesPrintMode>>,
    #[clap(
        long = "count-styles",
        help = "When printing styles, prefix with their sequence number"
    )]
    counted_styles: bool,
    #[clap(
        long = "print-terminal-schemes",
        help = "Print built-in terminal color schemes and quit"
    )]
    print_terminal_schemes: bool,
    // TODO: styles don't have descriptions yet
    // #[clap(long = "style-desc", help = "When printing styles, show descriptions")]
    // style_desc: bool,
}

#[derive(Debug, Clone, ValueEnum, Default)]
enum StylesPrintMode {
    #[default]
    Logical,
    Deps,
    List,
}

fn main() -> Result<(), GameError> {
    let _args = Args::parse();

    if _args.reset_config {
        Settings::reset_json_config(settings_path());
        return Ok(());
    }

    if _args.show_config_path {
        let settings_path = settings_path();
        if let Some(s) = settings_path.to_str() {
            println!("{}", s);
        } else {
            println!("{:?}", settings_path);
        }
        return Ok(());
    }

    if _args.debug_config {
        println!("{:#?}", Settings::load_json(settings_path(), true)?.read());
        return Ok(());
    }

    if _args.delete_data {
        let _ = std::fs::remove_file(save_data_path());
        return Ok(());
    }

    if let Some(mode) = _args.print_theme_options {
        print_style_options(mode.unwrap_or_default(), _args.counted_styles);
        return Ok(());
    }

    if _args.print_terminal_schemes {
        print_builtin_terminal_schemes();
        return Ok(());
    }

    better_panic::install();

    let mut app = App::empty(_args.read_only);
    let menu = MainMenu::new();
    app.activities_mut()
        .push(Activity::new_base_boxed("main menu", menu));

    #[cfg(feature = "updates")]
    updates::check(app.data_mut());

    app.run();

    Ok(())
}

fn print_style_options(mode: StylesPrintMode, counted: bool) {
    const TREE_INDENT: usize = 4;

    match mode {
        StylesPrintMode::Logical => {
            let theme_resolver = init_theme_resolver();
            let node = theme_resolver.to_logical_tree();
            let node_count = theme_resolver.as_map().len();
            if counted {
                node.print(TREE_INDENT, node_count.to_string().len() + 1, true, &mut 1);
            } else {
                node.print(TREE_INDENT, 0, false, &mut 1);
            }
        }
        StylesPrintMode::Deps => {
            let theme_resolver = init_theme_resolver();
            let node = theme_resolver.to_deps_tree();
            let node_count = theme_resolver.as_map().len();
            if counted {
                node.print(TREE_INDENT, node_count.to_string().len() + 1, true, &mut 1);
            } else {
                node.print(TREE_INDENT, 0, false, &mut 1);
            }
        }
        StylesPrintMode::List => {
            let theme_resolver = init_theme_resolver().to_map();
            let mut styles = theme_resolver.keys().collect::<Vec<_>>();
            styles.sort();
            for style in styles {
                println!("{}", style);
            }
        }
    }
}

fn print_builtin_terminal_schemes() {
    println!("Built-in terminal color schemes:");
    for name in tmaze::settings::theme::TerminalColorScheme::all_schemes() {
        println!("- {}", name);
    }
    println!("Credit to https://github.com/alacritty/alacritty-theme");
}
