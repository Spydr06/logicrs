use serde::{Serialize, Deserialize};

use crate::{renderer::{Renderable, DEFAULT_THEME}, simulator::Plot};
use std::cmp;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Selection {
    Single(u32),
    Many(Vec<u32>),
    Area((i32, i32), (i32, i32)),
    Connection {
        block_id: u32,
        output: u8,
        start: (i32, i32),
        position: (i32, i32),
    },
    None
}

impl Selection {
    pub fn connecting(&self) -> bool {
        matches!(self, Self::Connection {..})
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::None
    }
}

impl Renderable for Selection {
    fn render<R>(&self, renderer: &R, _data: &Plot) -> Result<(), R::Error>
        where R: crate::renderer::Renderer {

        match self {
            Self::Area(start, end) => {
                let position = (cmp::min(start.0, end.0), cmp::min(start.1, end.1));
                let size = (cmp::max(start.0, end.0) - position.0, cmp::max(start.1, end.1) - position.1);
                renderer.rectangle(position, size)
                    .set_line_width(1.)
                    .set_color(&DEFAULT_THEME.accent_bg_color)
                    .fill_preserve()?
                    .set_color(&DEFAULT_THEME.accent_fg_color)
                    .stroke().map(|_| ())
            }
            Self::Connection {start, position: end , ..} => {
                let offset = (
                    (start.0 + ((end.0 - start.0) as f32 * 0.7) as i32, start.1),
                    (end.0 + ((start.0 - end.0) as f32 * 0.7) as i32, end.1),
                );

                renderer.set_line_width(4.)
                    .set_color(&DEFAULT_THEME.disabled_bg_color)    
                    .move_to(*start)
                    .curve_to(offset.0, offset.1, *end)
                    .stroke().map(|_| ())
            }
            _ => Ok(())
        }
    }
}

pub trait SelectionField {
    fn selection(&self) -> &Selection;
    fn set_selection(&mut self, selection: Selection);

    fn unhighlight(&mut self);
    fn selected(&self) -> Vec<u32>;
    fn highlight_area(&mut self);
}
