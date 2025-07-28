use crate::renderer::{
    drawable::{Drawable, SizedDrawable},
    Frame,
};

pub enum Direction {
    Horizontal,
    Vertical,
}

pub enum AxisAlignment {
    Start,
    Center,
    End,
}

// pub struct Seq<S> {
//     pub items: Vec<Box<dyn SizedDrawable<S>>>,
//     pub spacing: i32,
//     pub direction: Direction,
//     pub cross_align: AxisAlignment,
// }
//
// impl<T> Seq<T> {
//     pub fn new(items: Vec<Box<dyn SizedDrawable<S>>>, direction: Direction) -> Self {
//         Self {
//             items,
//             spacing: 0,
//             direction,
//             cross_align: AxisAlignment::Start,
//         }
//     }
// }
//
// impl<T: SizedDrawable<S>, S: Clone> Drawable<S> for Seq<T> {
//     fn draw(&self, pos: cmaze::dims::Dims, frame: &mut impl Frame, styles: S) {
//         let mut x = pos.0;
//         let mut y = pos.1;
//         let container_size = self.size();
//
//         for item in &self.items {
//             let size = item.size();
//             let item_pos = cmaze::dims::Dims(x, y);
//
//             item.draw(item_pos, frame, styles.clone());
//
//             match self.direction {
//                 Direction::Horizontal => {
//                     x += size.0 + self.spacing;
//                     y = match self.cross_align {
//                         AxisAlignment::Start => pos.1,
//                         AxisAlignment::Center => pos.1 + (container_size.1 - size.1) / 2,
//                         AxisAlignment::End => pos.1 + container_size.1 - size.1,
//                     }
//                 }
//                 Direction::Vertical => {
//                     y += size.1 + self.spacing;
//                     x = match self.cross_align {
//                         AxisAlignment::Start => pos.0,
//                         AxisAlignment::Center => pos.0 + (container_size.0 - size.0) / 2,
//                         AxisAlignment::End => pos.0 + container_size.0 - size.0,
//                     };
//                 }
//             }
//         }
//     }
// }
//
// impl<T: SizedDrawable<S>, S: Clone> SizedDrawable<S> for Seq<T> {
//     fn size(&self) -> cmaze::dims::Dims {
//         let mut width = 0;
//         let mut height = 0;
//
//         for item in &self.items {
//             let size = item.size();
//             match self.direction {
//                 Direction::Horizontal => {
//                     width += size.0 + self.spacing;
//                     height = height.max(size.1);
//                 }
//                 Direction::Vertical => {
//                     width = width.max(size.0);
//                     height += size.1 + self.spacing;
//                 }
//             }
//         }
//
//         match self.direction {
//             Direction::Horizontal => width -= self.spacing,
//             Direction::Vertical => height -= self.spacing,
//         }
//
//         cmaze::dims::Dims(width, height)
//     }
// }
