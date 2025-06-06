use cmaze::dims::Dims;
use crossterm::event::{Event as TermEvent, KeyCode, KeyEvent};
use unicode_width::UnicodeWidthStr;

use crate::{
    app::{app::AppData, ActivityHandler, Change, Event},
    helpers::{not_release, LineDir},
    renderer::{drawable::Drawable, Cell, CellContent, Frame},
    settings::theme::Style,
    ui::{CapsuleText, Rect, Screen},
};

use super::theme::{StyleNode, Theme, ThemeResolver};

pub struct StyleBrowser {
    mode: Mode,
    resolver: ThemeResolver,
    search: String,
}

impl StyleBrowser {
    pub fn new(resolver: ThemeResolver) -> Self {
        let mut new = Self {
            mode: Mode::List(vec![]),
            resolver: resolver.clone(),
            search: String::new(),
        };

        new.use_logical();
        new
    }

    fn use_deps(&mut self) {
        self.mode = Mode::Deps(NodeItem::from_style_node(
            None,
            self.resolver.to_deps_tree(),
        ));
    }

    fn use_list(&mut self) {
        let mut list: Vec<_> = self.resolver.as_map().keys().collect();
        list.sort();
        self.mode = Mode::List(
            list.into_iter()
                .cloned()
                .map(|x| {
                    (
                        Item {
                            payload: x.clone(),
                            style: Some(x),
                        },
                        false,
                    )
                })
                .collect(),
        );
    }

    fn use_logical(&mut self) {
        self.mode = Mode::Logical(NodeItem::from_style_node(
            None,
            self.resolver.to_logical_tree(),
        ));
    }

    fn update_search(&mut self) {
        match &mut self.mode {
            Mode::Logical(node) => {
                node.match_search_pattern(&self.search, Some(false));
            }
            Mode::Deps(node) => {
                node.match_search_pattern(&self.search, None);
            }
            Mode::List(items) => {
                for (item, hidden) in items {
                    *hidden = !item.payload.contains(&self.search);
                }
            }
        }
    }
}

impl ActivityHandler for StyleBrowser {
    fn update(&mut self, events: Vec<Event>, _: &mut AppData) -> Option<Change> {
        for event in events {
            match event {
                Event::Term(TermEvent::Key(KeyEvent { code, kind, .. })) if not_release(kind) => {
                    match code {
                        KeyCode::Esc => return Some(Change::pop_top()),
                        KeyCode::Tab => match &self.mode {
                            Mode::Logical(_) => self.use_deps(),
                            Mode::Deps(_) => self.use_list(),
                            Mode::List(_) => self.use_logical(),
                        },
                        KeyCode::BackTab => match &self.mode {
                            Mode::Logical(_) => self.use_list(),
                            Mode::Deps(_) => self.use_logical(),
                            Mode::List(_) => self.use_deps(),
                        },
                        KeyCode::Char(c) => {
                            self.search.push(c);
                            self.update_search();
                        }
                        KeyCode::Backspace => {
                            self.search.pop();
                            self.update_search();
                        }

                        _ => {}
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn screen(&self) -> &dyn Screen {
        self
    }
}

impl Screen for StyleBrowser {
    fn draw(&self, frame: &mut Frame, theme: &Theme) -> std::io::Result<()> {
        const INDENT: i32 = 4;

        let [border, text, dim, background] =
            theme.extract(["sb.border", "sb.text", "sb.dim", "sb.background"]);

        let margin = Dims(4, 1);
        Rect::new(margin, frame.size - margin - Dims(1, 1)).draw(Dims(0, 0), frame, border);

        let mut inner_frame = Frame::new(frame.size - margin * 2 - Dims(2, 2));
        inner_frame.fill(Cell::Content(CellContent {
            character: ' ',
            width: 1,
            style: background.into(),
        }));
        {
            if self.search.is_empty() {
                inner_frame.draw(Dims(1, 0), "<Search>", dim);
            } else {
                inner_frame.draw(Dims(1, 0), self.search.as_str(), text);
            }

            const TABS: &[(&str, fn(&Mode) -> bool)] = &[
                ("By name", |x| matches!(x, Mode::Logical(_))),
                ("Inheritance", |x| matches!(x, Mode::Deps(_))),
                ("List", |x| matches!(x, Mode::List(_))),
            ];

            let tabs_width = TABS
                .iter()
                .map(|(name, _)| name.width() as i32 + 3)
                .sum::<i32>();

            // i've no why -2 is needed here, but it's cutoff without it
            let mut xoof = inner_frame.size.0 - tabs_width - 6;
            for (name, is_mode) in TABS {
                if is_mode(&self.mode) {
                    inner_frame.draw(
                        Dims(xoof, 0),
                        CapsuleText(format!(" {name} ")),
                        text.invert(),
                    );
                } else {
                    inner_frame.draw(Dims(xoof, 0), format!("  {name}  "), text);
                };
                xoof += name.width() as i32 + 5;
            }
        }

        inner_frame.draw(
            Dims(0, 1),
            LineDir::Horizontal
                .round()
                .to_string()
                .repeat(inner_frame.size.0 as usize),
            border,
        );

        fn print_node(
            frame: &mut Frame,
            node: &NodeItem,
            pos: Dims,
            style: Style,
            theme: &Theme,
        ) -> i32 {
            if node.hidden {
                return 0;
            }
            frame.draw(
                pos,
                node.item
                    .as_ref()
                    .expect("non-root node must have payload")
                    .payload
                    .as_str(),
                style,
            );

            if let Some(node_style) = node.item.as_ref().and_then(|i| i.style.as_ref()) {
                let node_style = theme.get(node_style);
                let style_text = match (node_style.fg, node_style.bg) {
                    (Some(fg), Some(gb)) => format!("{} on {}", fg.as_text(), gb.as_text()),
                    (Some(fg), None) => format!("{fg}", fg = fg.as_text()),
                    (None, Some(bg)) => format!("on {bg}", bg = bg.as_text()),
                    (None, None) => String::new(),
                };
                let width = style_text.width() as i32 + 1;

                frame.draw(
                    Dims(frame.size.0 - width, pos.1),
                    style_text.as_str(),
                    node_style,
                );
            }

            let mut yoff = 0;
            for child in &node.children {
                yoff += print_node(frame, child, pos + Dims(INDENT, yoff + 1), style, theme);
            }
            yoff + 1
        }

        let mut yoff = 2;
        const LEFT_MARGIN: i32 = 1;
        match &self.mode {
            Mode::Logical(node) | Mode::Deps(node) => {
                for child in &node.children {
                    yoff += print_node(
                        &mut inner_frame,
                        child,
                        Dims(LEFT_MARGIN, yoff),
                        text,
                        theme,
                    );
                }
            }
            Mode::List(items) => {
                for (i, (item, _)) in items.iter().filter(|(_, h)| !h).enumerate() {
                    inner_frame.draw(
                        Dims(LEFT_MARGIN, i as i32 + yoff),
                        item.payload.as_str(),
                        text,
                    );
                }
            }
        }

        frame.draw(margin + Dims(1, 1), &inner_frame, ());

        Ok(())
    }
}

#[derive(Debug)]
enum Mode {
    Logical(NodeItem),
    Deps(NodeItem),
    List(Vec<(Item, bool)>),
}

#[derive(Debug)]
struct NodeItem {
    item: Option<Item>,
    hidden: bool,
    children: Vec<NodeItem>,
}

#[derive(Debug)]
struct Item {
    payload: String,
    style: Option<String>,
}

impl NodeItem {
    fn from_style_node(root: Option<Item>, style_node: StyleNode<'_>) -> NodeItem {
        let mut node = NodeItem {
            item: root,
            children: Vec::new(),
            hidden: false,
        };

        node.children.reserve(style_node.map.len());
        for (key, value) in style_node.map {
            node.children.push(Self::from_style_node(
                Some(Item {
                    payload: key.to_string(),
                    style: value.style.map(Into::into),
                }),
                value,
            ));
        }

        node
    }

    fn match_search_pattern(&mut self, pattern: &str, propagate_down: Option<bool>) -> bool {
        self.hidden = true;
        if let Some(item) = &self.item {
            if item.payload.contains(pattern) {
                self.hidden = false;
            }
        }

        let to_propagate = propagate_down.map(|down| !self.hidden || down);
        for child in &mut self.children {
            if child.match_search_pattern(pattern, to_propagate) {
                self.hidden = false;
            }
        }

        let show_primary = !self.hidden;
        self.hidden = self.hidden && !propagate_down.unwrap_or(false);

        show_primary
    }
}

pub fn style_browser_theme_resolver() -> ThemeResolver {
    let mut resolver = ThemeResolver::new();
    resolver
        .link("sb.border", "border")
        .link("sb.text", "text")
        .link("sb.dim", "dim")
        .link("sb.background", "background");
    resolver
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::theme::ThemeResolver;

    #[test]
    fn style_browser_general() {
        let resolver = ThemeResolver::new();
        let mut style_browser = StyleBrowser::new(resolver);
        style_browser.use_logical();
        style_browser.use_deps();
        style_browser.use_list();
    }

    #[test]
    fn style_browser_node_item_search() {
        let resolver = ThemeResolver::new();
        let mut style_browser = StyleBrowser::new(resolver);
        style_browser.use_logical();

        let mut node = NodeItem {
            item: None,
            hidden: false,
            children: vec![
                NodeItem {
                    item: Some(Item {
                        payload: "test".to_string(),
                        style: None,
                    }),
                    children: vec![NodeItem {
                        item: Some(Item {
                            payload: "test.child".to_string(),
                            style: None,
                        }),
                        children: vec![],
                        hidden: false,
                    }],
                    hidden: false,
                },
                NodeItem {
                    item: Some(Item {
                        payload: "example".to_string(),
                        style: None,
                    }),
                    children: vec![],
                    hidden: false,
                },
            ],
        };

        assert!(node.match_search_pattern("test", None));
        assert!(node.match_search_pattern("example", None));
        assert!(node.match_search_pattern("child", None));
        assert!(node.match_search_pattern("", None));
        assert!(!node.match_search_pattern("unknown", None));
    }
}
