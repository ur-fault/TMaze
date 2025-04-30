use cmaze::dims::Dims;
use crossterm::event::{Event as TermEvent, KeyCode, KeyEvent};
use unicode_width::UnicodeWidthStr;

use crate::{
    app::{app::AppData, ActivityHandler, Change, Event},
    renderer::{drawable::Drawable, Frame},
    settings::theme::Style,
    ui::{Rect, Screen},
};

use super::theme::{StyleNode, Theme, ThemeResolver};

pub struct StyleBrowser {
    mode: Mode,
}

impl StyleBrowser {
    pub fn new(resolver: &ThemeResolver) -> Self {
        // panic!("{:#?}", resolver.to_logical_tree());
        Self {
            mode: Mode::Logical(NodeItem::from_style_node(
                Item {
                    payload: "<root>".into(),
                    style: None,
                },
                resolver.to_logical_tree(),
            )),
        }
    }

    fn use_deps(&mut self, app_data: &mut AppData) {
        self.mode = Mode::Deps(NodeItem::from_style_node(
            Item {
                payload: "<root>".into(),
                style: None,
            },
            app_data.theme_resolver.to_deps_tree(),
        ));
    }

    fn use_list(&mut self, app_data: &mut AppData) {
        let mut list: Vec<_> = app_data.theme_resolver.as_map().keys().collect();
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

    fn use_logical(&mut self, app_data: &mut AppData) {
        self.mode = Mode::Logical(NodeItem::from_style_node(
            Item {
                payload: "<root>".into(),
                style: None,
            },
            app_data.theme_resolver.to_logical_tree(),
        ));
    }
}

impl ActivityHandler for StyleBrowser {
    fn update(&mut self, events: Vec<Event>, app_data: &mut AppData) -> Option<Change> {
        for event in events {
            match event {
                Event::Term(TermEvent::Key(KeyEvent { code, .. })) => match code {
                    KeyCode::Esc => return Some(Change::pop_top()),
                    KeyCode::Tab => match &self.mode {
                        Mode::Logical(_) => self.use_deps(app_data),
                        Mode::Deps(_) => self.use_list(app_data),
                        Mode::List(_) => self.use_logical(app_data),
                    },
                    KeyCode::BackTab => match &self.mode {
                        Mode::Logical(_) => self.use_list(app_data),
                        Mode::Deps(_) => self.use_logical(app_data),
                        Mode::List(_) => self.use_deps(app_data),
                    },
                    _ => {}
                },
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
            for (name, mode) in &TABS {
                let style = if mode(&self.mode) {
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
                frame.draw(pos, node.item.payload.as_str(), style);
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
    item: Item,
    children: Vec<NodeItem>,
    hidden: bool,
}

impl NodeItem {
    fn from_style_node(root: Item, style_node: StyleNode<'_>) -> NodeItem {
        let mut node = NodeItem {
            item: root,
            children: Vec::new(),
            hidden: false,
        };

        for (key, value) in style_node.map {
            node.children.push(Self::from_style_node(
                Item {
                    payload: key.to_string(),
                    style: value.style.map(Into::into),
                },
                value,
            ));
        }

        node
    }
}

#[derive(Debug)]
struct Item {
    payload: String,
    style: Option<String>,
}
