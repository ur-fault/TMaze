use cmaze::dims::Dims;
use crossterm::{
    event::{Event as TermEvent, KeyCode, KeyEvent},
    style::{Attribute, Attributes},
};
use log::debug;
use unicode_width::UnicodeWidthStr;

use crate::{
    app::{app::AppData, ActivityHandler, Change, Event},
    helpers::{not_release, LineDir},
    renderer::{drawable::Drawable, Cell, CellContent, Frame},
    settings::theme::Style,
    ui::{CapsuleText, Rect, Screen},
};

use super::theme::{StyleNode, Theme, ThemeResolver};

const CONTENT_MARGIN: Dims = Dims(4, 1);
const LEFT_MARGIN: i32 = 1;
const RIGHT_MARGIN: i32 = 1;

pub struct StyleBrowser {
    mode: Mode,
    resolver: ThemeResolver,
    search: String,
    selected_index: usize,
    scroll: i32,
    list_height: i32,
}

impl StyleBrowser {
    pub fn new(resolver: ThemeResolver) -> Self {
        let mut new = Self {
            mode: Mode::List(vec![]),
            resolver: resolver.clone(),
            search: String::new(),
            selected_index: 0,
            scroll: 0,
            list_height: 0,
        };

        new.use_logical();
        new
    }

    fn use_deps(&mut self) {
        self.mode = Mode::Deps(NodeItem::from_style_node(
            None,
            self.resolver.to_deps_tree(),
            &mut 0,
        ));
        self.reset_selected();
    }

    fn use_list(&mut self) {
        let mut list: Vec<_> = self.resolver.as_map().keys().collect();
        list.sort();
        let style_list = list
            .into_iter()
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
            .collect();
        self.mode = Mode::List(style_list);
        self.reset_selected();
    }

    fn use_logical(&mut self) {
        self.mode = Mode::Logical(NodeItem::from_style_node(
            None,
            self.resolver.to_logical_tree(),
            &mut 0,
        ));
        self.reset_selected();
    }

    fn reset_selected(&mut self) {
        self.selected_index = 0;
        self.scroll = 0;
    }

    fn update_search(&mut self) {
        fn select_closest_node(index: usize, node: &NodeItem) -> usize {
            if node
                .iter_visible()
                .filter(|(n, _)| n.item_index == index)
                .next()
                .is_some()
            {
                return index;
            }

            match node
                .iter_visible()
                .map(|(n, _)| ((n.item_index as i32).abs_diff(index as i32), n.item_index))
                .min_by_key(|(diff, _)| *diff)
            {
                Some((_, index)) => index,
                None => 0,
            }
        }
        match &mut self.mode {
            Mode::Logical(node) => {
                node.match_search_pattern(&self.search, Some(false));
                self.selected_index = select_closest_node(self.selected_index, node);
            }
            Mode::Deps(node) => {
                node.match_search_pattern(&self.search, None);
                self.selected_index = select_closest_node(self.selected_index, node);
            }
            Mode::List(items) => {
                for (item, hidden) in items.iter_mut() {
                    *hidden = !item.payload.contains(&self.search);
                }

                // find closest item index to selected
                if items
                    .iter()
                    .enumerate()
                    .filter(|(_, (_, hidden))| !hidden)
                    .filter(|(index, (_, _))| *index == self.selected_index)
                    .next()
                    .is_none()
                {
                    match items
                        .iter()
                        .enumerate()
                        .filter(|(_, (_, hidden))| !hidden)
                        .map(|(index, _)| {
                            ((index as i32).abs_diff(self.selected_index as i32), index)
                        })
                        .min_by_key(|(diff, _)| *diff)
                    {
                        Some((_, index)) => self.selected_index = index,
                        None => {
                            self.selected_index = 0;
                        }
                    }
                }
            }
        }

        self.scroll_to_selected();
    }

    fn update_selected(&mut self, up: bool) {
        match &self.mode {
            Mode::Logical(node) | Mode::Deps(node) => {
                if node.is_empty() {
                    self.selected_index = 0;
                    return;
                }

                let iter = node.iter();

                let iter = if !up {
                    iter.skip(self.selected_index + 1).find(|(n, _)| !n.hidden)
                } else {
                    // oh boy, this is a pretty ugly hack
                    iter.collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .skip(node.len() - self.selected_index)
                        .find(|(n, _)| !n.hidden)
                };

                self.selected_index = iter.map_or(self.selected_index, |(n, _)| n.item_index);
            }
            Mode::List(items) => {
                if items.is_empty() {
                    self.selected_index = 0;
                    return;
                }

                let iter = items.iter().enumerate();

                let iter = if !up {
                    iter.skip(self.selected_index + 1)
                        .find(|(_, (_, hidden))| !hidden)
                } else {
                    iter.rev()
                        .skip(items.len() - self.selected_index)
                        .find(|(_, (_, hidden))| !hidden)
                };

                self.selected_index = iter.map_or(self.selected_index, |(index, _)| index);
            }
        }

        self.scroll_to_selected();
    }

    fn scroll_to_selected(&mut self) {
        let window = (self.scroll, self.scroll + self.list_height);
        match &self.mode {
            Mode::Logical(node) | Mode::Deps(node) => {
                let visible_index = node
                    .iter_visible()
                    .position(|(n, _)| n.item_index == self.selected_index);

                let Some(visible_index) = visible_index else {
                    self.scroll = 0;
                    return;
                };

                if visible_index < window.0 as usize {
                    self.scroll = visible_index as i32;
                } else if visible_index >= window.1 as usize {
                    self.scroll = visible_index as i32 - self.list_height + 1;
                }

                debug!("scroll: {}, selected index: {}, visible index: {visible_index}, window: {window:?}", self.scroll, self.selected_index);
            }
            Mode::List(items) => {
                let visible_index = items
                    .iter()
                    .enumerate()
                    .filter(|(_, (_, hidden))| !hidden)
                    .map(|(index, _)| index)
                    .position(|index| index == self.selected_index);

                let Some(visible_index) = visible_index else {
                    self.scroll = 0;
                    return;
                };

                if visible_index < window.0 as usize {
                    self.scroll = visible_index as i32;
                } else if visible_index >= window.1 as usize {
                    self.scroll = visible_index as i32 - self.list_height + 1;
                }

                debug!("scroll: {}, selected index: {}, visible index: {visible_index}, window: {window:?}", self.scroll, self.selected_index);
            }
        }
    }

    fn len(&self) -> usize {
        match &self.mode {
            Mode::Logical(node) | Mode::Deps(node) => node.len(),
            Mode::List(items) => items.len(),
        }
    }

    fn visible_len(&self) -> usize {
        match &self.mode {
            Mode::Logical(node) | Mode::Deps(node) => node.visible_len(),
            Mode::List(items) => items.iter().filter(|(_, hidden)| !hidden).count(),
        }
    }
}

impl ActivityHandler for StyleBrowser {
    fn update(&mut self, events: Vec<Event>, data: &mut AppData) -> Option<Change> {
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
                        KeyCode::Down | KeyCode::Up => {
                            self.update_selected(matches!(code, KeyCode::Up));
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn screen(&mut self) -> &mut dyn Screen {
        self
    }
}

impl Screen for StyleBrowser {
    fn draw(&mut self, frame: &mut Frame, theme: &Theme) -> std::io::Result<()> {
        const INDENT: i32 = 4;

        let [border, text, dim, background] =
            theme.extract(["sb.border", "sb.text", "sb.dim", "sb.background"]);

        Rect::new(CONTENT_MARGIN, frame.size - CONTENT_MARGIN - Dims(1, 1)).draw(
            Dims(0, 0),
            frame,
            border,
        );

        let mut inner_frame = Frame::new(frame.size - CONTENT_MARGIN * 2 - Dims(2, 2));
        inner_frame.fill(Cell::Content(CellContent {
            character: ' ',
            width: 1,
            style: background.into(),
        }));
        self.list_height = inner_frame.size.1 - 2;

        {
            if self.search.is_empty() {
                inner_frame.draw(Dims(1, 0), "<Search>", dim);
            } else {
                inner_frame.draw(Dims(1, 0), self.search.as_str(), text);
            }

            const TABS: [(&str, fn(&Mode) -> bool); 3] = [
                ("By name", |x| matches!(x, Mode::Logical(_))),
                ("Inheritance", |x| matches!(x, Mode::Deps(_))),
                ("List", |x| matches!(x, Mode::List(_))),
            ];

            let tabs_width = TABS
                .iter()
                .map(|(name, _)| name.width() as i32 + 3)
                .sum::<i32>();

            // i've no why -2 is needed here, but it's cut off without it
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

        fn render_style(style: &str, theme: &Theme) -> (String, Style, i32) {
            let style = theme.get(style);
            let text = match (style.fg, style.bg) {
                (Some(fg), Some(gb)) => format!("{} on {}", fg.as_text(), gb.as_text()),
                (Some(fg), None) => format!("{fg}", fg = fg.as_text()),
                (None, Some(bg)) => format!("on {bg}", bg = bg.as_text()),
                (None, None) => String::new(),
            };
            let width = text.width() as i32;
            (text, style, width)
        }

        let yoff = 2;
        match &self.mode {
            Mode::Logical(node) | Mode::Deps(node) => {
                for (index, (node, depth)) in node
                    .iter_visible()
                    .skip(self.scroll.max(0) as usize)
                    .enumerate()
                {
                    let pos = Dims(LEFT_MARGIN + depth as i32 * INDENT, yoff + index as i32);
                    inner_frame.draw(
                        pos,
                        format!(
                            "{}:{}",
                            node.item_index,
                            node.item
                                .as_ref()
                                .expect("non-root node must have payload")
                                .payload
                                .as_str(),
                        ),
                        if node.hidden { dim } else { text },
                    );

                    if let Some(node_style) = node.item.as_ref().and_then(|i| i.style.as_ref()) {
                        let (style_text, node_style, width) = render_style(node_style, theme);

                        inner_frame.draw(
                            Dims(frame.size.0 - width - RIGHT_MARGIN, pos.1),
                            style_text.as_str(),
                            node_style,
                        );
                    }

                    if self.selected_index == node.item_index {
                        for x in 0..inner_frame.size.0 {
                            if let Some(cell) = inner_frame.try_get_mut(Dims(x, pos.1)) {
                                match cell {
                                    c @ Cell::Empty => {
                                        *c = Cell::Content(CellContent {
                                            character: ' ',
                                            width: 1,
                                            style: crossterm::style::ContentStyle {
                                                attributes: Attributes::from(Attribute::Underlined),
                                                ..crossterm::style::ContentStyle::default()
                                            },
                                        })
                                    }
                                    Cell::Content(c) => {
                                        c.style.attributes.extend(Attribute::Underlined.into())
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Mode::List(items) => {
                let mut current = 0;
                for (index, (item, hidden)) in items.iter().enumerate() {
                    if *hidden || index < self.scroll as usize {
                        continue;
                    }

                    if let Some(item_style) = item.style.as_ref() {
                        let (style_text, node_style, width) = render_style(&item_style, theme);

                        inner_frame.draw(
                            Dims(inner_frame.size.0 - width - RIGHT_MARGIN, current + yoff),
                            style_text.as_str(),
                            node_style,
                        );
                    }

                    inner_frame.draw(
                        Dims(LEFT_MARGIN, current + yoff),
                        item.payload.as_str(),
                        text,
                    );

                    if self.selected_index == index {
                        for x in 0..inner_frame.size.0 {
                            if let Some(cell) = inner_frame.try_get_mut(Dims(x, current + yoff)) {
                                match cell {
                                    c @ Cell::Empty => {
                                        *c = Cell::Content(CellContent {
                                            character: ' ',
                                            width: 1,
                                            style: crossterm::style::ContentStyle {
                                                attributes: Attributes::from(Attribute::Underlined),
                                                ..crossterm::style::ContentStyle::default()
                                            },
                                        })
                                    }
                                    Cell::Content(c) => {
                                        c.style.attributes.extend(Attribute::Underlined.into())
                                    }
                                }
                            }
                        }
                    }

                    current += 1;
                }
            }
        }

        frame.draw(CONTENT_MARGIN + Dims(1, 1), &inner_frame, ());

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
    item_index: usize,
}

#[derive(Debug)]
struct Item {
    payload: String,
    style: Option<String>,
}

impl NodeItem {
    fn from_style_node(
        root: Option<Item>,
        style_node: StyleNode<'_>,
        index: &mut usize,
    ) -> NodeItem {
        let mut node = NodeItem {
            item: root,
            children: Vec::new(),
            hidden: false,
            item_index: *index,
        };

        if node.item.is_some() {
            *index += 1;
        }

        node.children.reserve(style_node.map.len());
        for (key, value) in style_node.map {
            node.children.push(Self::from_style_node(
                Some(Item {
                    payload: key.to_string(),
                    style: value.style.map(Into::into),
                }),
                value,
                index,
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

    fn len(&self) -> usize {
        fn inner(node: &NodeItem) -> usize {
            let mut count = 1; // count this node
            for child in &node.children {
                count += child.len();
            }
            count
        }

        inner(self) - if self.item.is_none() { 1 } else { 0 }
    }

    fn visible_len(&self) -> usize {
        fn inner(node: &NodeItem) -> usize {
            if node.hidden {
                return 0;
            }
            let mut count = 1; // count this node
            for child in &node.children {
                count += inner(child);
            }
            count
        }

        inner(self)
            - if self.item.is_none() && !self.hidden {
                1
            } else {
                0
            }
    }

    fn is_empty(&self) -> bool {
        self.children.is_empty() && self.item.is_none()
    }

    fn iter(&self) -> NodeItemIter {
        NodeItemIter {
            node: self,
            index: 0,
            child: None,
            root: true,
            visible_only: false,
        }
    }

    fn iter_visible(&self) -> NodeItemIter {
        NodeItemIter {
            node: self,
            index: 0,
            child: None,
            root: true,
            visible_only: true,
        }
    }
}

#[derive(Debug)]
struct NodeItemIter<'a> {
    node: &'a NodeItem,
    index: usize,
    child: Option<Box<NodeItemIter<'a>>>,
    root: bool,
    visible_only: bool,
}

impl<'a> Iterator for NodeItemIter<'a> {
    type Item = (&'a NodeItem, usize);

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: this implementation should not allocate on every non-leaf node

        if self.visible_only && self.node.hidden {
            return None;
        }

        if self.node.children.is_empty() {
            if self.index == 0 {
                self.index = 1;
                if !self.root {
                    return Some((self.node, 0));
                }
            }

            return None;
        }

        let child = match self.child {
            Some(ref mut child) => child,
            None => {
                self.child = Some(Box::new(NodeItemIter {
                    node: &self.node.children[0],
                    index: 0,
                    child: None,
                    root: false,
                    visible_only: self.visible_only,
                }));
                match self.root {
                    true => self.child.as_mut().unwrap(),
                    false => return Some((self.node, 0)),
                }
            }
        };

        if let Some((next, depth)) = child.next() {
            return Some((next, depth + if self.root { 0 } else { 1 }));
        }

        // to handle visible children after hidden ones, we need this loop
        loop {
            self.index += 1;
            if self.index >= self.node.children.len() {
                return None;
            }

            let next_child = &self.node.children[self.index];
            self.child = Some(Box::new(NodeItemIter {
                node: next_child,
                index: 0,
                child: None,
                root: false,
                visible_only: self.visible_only,
            }));

            match self
                .child
                .as_mut()
                .unwrap()
                .next()
                .map(|(n, d)| (n, d + if self.root { 0 } else { 1 }))
            {
                Some(item) => return Some(item),
                None => {}
            }
        }
    }

    fn count(self) -> usize {
        if self.visible_only {
            self.node.visible_len()
        } else {
            self.node.len()
        }
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
        let mut node = NodeItem {
            item: None,
            hidden: false,
            item_index: 0,
            children: vec![
                NodeItem {
                    item: Some(Item {
                        payload: "test".to_string(),
                        style: None,
                    }),
                    hidden: false,
                    item_index: 1,
                    children: vec![NodeItem {
                        item: Some(Item {
                            payload: "test.child".to_string(),
                            style: None,
                        }),
                        hidden: false,
                        item_index: 2,
                        children: vec![],
                    }],
                },
                NodeItem {
                    item: Some(Item {
                        payload: "example".to_string(),
                        style: None,
                    }),
                    hidden: false,
                    item_index: 3,
                    children: vec![],
                },
            ],
        };

        node.match_search_pattern("test", Some(false));
        assert!(!node.hidden);
        assert!(!node.children[0].hidden);
        assert!(node.children[1].hidden);

        node.match_search_pattern("child", Some(false));
        assert!(!node.hidden);
        assert!(!node.children[0].hidden);
        assert!(node.children[1].hidden);

        node.match_search_pattern("example", Some(false));
        assert!(!node.hidden);
        assert!(node.children[0].hidden);
        assert!(!node.children[1].hidden);

        node.match_search_pattern("fail", Some(false));
        assert!(node.hidden);
        assert!(node.children[0].hidden);
        assert!(node.children[1].hidden);

        assert!(node.match_search_pattern("test", None));
        assert!(node.match_search_pattern("example", None));
        assert!(node.match_search_pattern("child", None));
        assert!(node.match_search_pattern("", None));
        assert!(!node.match_search_pattern("unknown", None));
    }

    #[test]
    fn style_browser_node_item_iter() {
        let resolver = ThemeResolver::new();
        let mut style_browser = StyleBrowser::new(resolver);
        style_browser.use_logical();

        let node = NodeItem {
            item: None,
            hidden: false,
            item_index: 0,
            children: vec![
                NodeItem {
                    item: Some(Item {
                        payload: "test".to_string(),
                        style: None,
                    }),
                    hidden: false,
                    item_index: 1,
                    children: vec![NodeItem {
                        item: Some(Item {
                            payload: "test.child".to_string(),
                            style: None,
                        }),
                        hidden: false,
                        item_index: 2,
                        children: vec![],
                    }],
                },
                NodeItem {
                    item: Some(Item {
                        payload: "example".to_string(),
                        style: None,
                    }),
                    hidden: false,
                    item_index: 3,
                    children: vec![],
                },
            ],
        };

        dbg!(node.iter().collect::<Vec<_>>());
        let mut iter = node.iter();

        fn transform_next(next: Option<(&NodeItem, usize)>) -> (usize, &str) {
            next.map(|(node, depth)| (depth, node.item.as_ref().unwrap().payload.as_str()))
                .unwrap()
        }

        assert_eq!(transform_next(iter.next()), (0, "test"));
        assert_eq!(transform_next(iter.next()), (1, "test.child"));
        assert_eq!(transform_next(iter.next()), (0, "example"));
        assert!(iter.next().is_none());

        assert_eq!(node.len(), 3);
        assert_eq!(node.iter().count(), 3);
    }

    #[test]
    fn style_browser_node_item_iter_visible() {
        let resolver = ThemeResolver::new();
        let mut style_browser = StyleBrowser::new(resolver);
        style_browser.use_logical();

        let node = NodeItem {
            item: None,
            hidden: false,
            item_index: 0,
            children: vec![
                NodeItem {
                    item: Some(Item {
                        payload: "visible".to_string(),
                        style: None,
                    }),
                    hidden: false,
                    item_index: 1,
                    children: vec![NodeItem {
                        item: Some(Item {
                            payload: "visible.child".to_string(),
                            style: None,
                        }),
                        hidden: false,
                        item_index: 2,
                        children: vec![],
                    }],
                },
                NodeItem {
                    item: Some(Item {
                        payload: "hidden".to_string(),
                        style: None,
                    }),
                    hidden: true,
                    item_index: 3,
                    children: vec![NodeItem {
                        item: Some(Item {
                            payload: "hidden.child".to_string(),
                            style: None,
                        }),
                        hidden: true,
                        item_index: 4,
                        children: vec![],
                    }],
                },
            ],
        };

        dbg!(node.iter_visible().collect::<Vec<_>>());
        let mut iter = node.iter_visible();

        fn transform_next(next: Option<(&NodeItem, usize)>) -> (usize, &str) {
            next.map(|(node, depth)| (depth, node.item.as_ref().unwrap().payload.as_str()))
                .unwrap()
        }

        assert_eq!(transform_next(iter.next()), (0, "visible"));
        assert_eq!(transform_next(iter.next()), (1, "visible.child"));
        assert!(iter.next().is_none());

        assert_eq!(node.len(), 4);
        assert_eq!(node.visible_len(), 2);
    }
}
