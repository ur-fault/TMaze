use cmaze::dims::Dims;
use crossterm::event::{Event as TermEvent, KeyCode, KeyEvent};
use unicode_width::UnicodeWidthStr;

use crate::{
    app::{app::AppData, ActivityHandler, Change, Event},
    helpers::not_release,
    renderer::{drawable::Drawable, Frame},
    settings::theme::Style,
    ui::{Rect, Screen},
};

use super::theme::{StyleNode, Theme, ThemeResolver};

pub struct StyleBrowser {
    mode: Mode,
    resolver: ThemeResolver,
}

impl StyleBrowser {
    pub fn new(resolver: ThemeResolver) -> Self {
        // panic!("{:#?}", resolver.to_logical_tree());
        let mut new = Self {
            mode: Mode::List(vec![]),
            resolver: resolver.clone(),
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
                .map(|x| Item {
                    payload: x.clone(),
                    style: Some(x),
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

        let border = theme["border"];
        let text = theme["text"];

        let margin = Dims(3, 1) * 2;
        Rect::new(margin, frame.size - margin).draw(Dims(0, 0), frame, border);

        {
            const TABS: [(&str, fn(&Mode) -> bool); 3] = [
                ("Logical", |x| matches!(x, Mode::Logical(_))),
                ("Inheritance", |x| matches!(x, Mode::Deps(_))),
                ("List", |x| matches!(x, Mode::List(_))),
            ];

            let mut xoof = 0;
            for (name, is_mode) in &TABS {
                let style = if is_mode(&self.mode) {
                    text.invert()
                } else {
                    text
                };
                frame.draw(Dims(xoof, 0), format!(" {name} "), style);
                xoof += name.width() as i32 + 3;
            }
        }

        let mut inner_frame = Frame::new(frame.size - margin * 2 - Dims(2, 2));
        {
            fn print_node(frame: &mut Frame, node: &NodeItem, pos: Dims, style: Style) -> i32 {
                frame.draw(
                    pos,
                    node.item
                        .as_ref()
                        .map(|item| item.payload.as_str())
                        .unwrap_or("<root>"),
                    style,
                );
                let mut yoff = 0;
                for child in &node.children {
                    yoff += print_node(frame, child, pos + Dims(INDENT, yoff + 1), style);
                }
                yoff + 1
            }

            match &self.mode {
                Mode::Logical(node) | Mode::Deps(node) => {
                    let mut yoff = 0;
                    for child in &node.children {
                        yoff += print_node(&mut inner_frame, child, Dims(0, yoff), text);
                    }
                }
                Mode::List(items) => {
                    for (i, item) in items.iter().enumerate() {
                        inner_frame.draw(Dims(0, i as i32), item.payload.as_str(), text);
                    }
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
    List(Vec<Item>),
}

#[derive(Debug)]
struct NodeItem {
    item: Option<Item>,
    hidden: bool,
    children: Vec<NodeItem>,
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

    fn match_search_pattern(&mut self, pattern: &str) -> bool {
        self.hidden = true;
        if let Some(item) = &self.item {
            if item.payload.contains(pattern) {
                self.hidden = false;
            }
        }

        for child in &mut self.children {
            if child.match_search_pattern(pattern) {
                self.hidden = false;
            }
        }

        !self.hidden
    }
}

#[derive(Debug)]
struct Item {
    payload: String,
    style: Option<String>,
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

        assert!(node.match_search_pattern("test"));
        assert!(node.match_search_pattern("example"));
        assert!(node.match_search_pattern("child"));
        assert!(node.match_search_pattern(""));
        assert!(!node.match_search_pattern("unknown"));
    }
}
