use std::collections::BTreeMap;

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
        print_style_options(mode.unwrap_or_default());
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

fn print_style_options(mode: StylesPrintMode) {
    const TREE_INDENT: usize = 4;

    match mode {
        StylesPrintMode::List => {
            let theme_resolver = init_theme_resolver().to_map();
            let mut styles = theme_resolver.keys().collect::<Vec<_>>();
            styles.sort();
            for style in styles {
                println!("{}", style);
            }
        }
        StylesPrintMode::Deps => todo!(),
        StylesPrintMode::Logical => {
            #[derive(Debug)]
            struct Node<'a>(BTreeMap<&'a str, Node<'a>>);

            impl<'a> Node<'a> {
                fn new() -> Self {
                    Self(BTreeMap::new())
                }

                fn add(&mut self, rem_segs: &[&'a str]) {
                    if rem_segs.is_empty() {
                        return;
                    }
                    let seg = rem_segs[0];
                    let node = self.0.entry(seg).or_insert_with(Node::new);
                    if rem_segs.len() > 1 {
                        node.add(&rem_segs[1..]);
                    }
                }

                fn print(&self, depth: usize) {
                    for (key, node) in &self.0 {
                        println!("{}{}", " ".repeat(depth), key);
                        node.print(depth + TREE_INDENT);
                    }
                }
            }

            let mut node = Node::new();
            let theme_resolver = init_theme_resolver().to_map();
            for style in theme_resolver.keys() {
                let segs = style.split('.').collect::<Vec<_>>();
                node.add(&segs);
            }
            node.print(0);
        }
    }
}
