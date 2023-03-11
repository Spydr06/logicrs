use serde::{Serialize, Deserialize};

use crate::{renderer::{Renderable, COLOR_THEME, vector::Vector2}, simulator::{Plot, BlockID}};
use std::cmp;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Selection {
    Single(BlockID, Vector2<i32>),
    Many(Vec<BlockID>),
    Area(Vector2<i32>, Vector2<i32>),
    MouseEvent(BlockID),
    Connection {
        block_id: BlockID,
        output: u8,
        start: Vector2<i32>,
        position: Vector2<i32>,
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
                let position = cmp::min(start, end);
                let size = *cmp::max(start, end) - *position;
                renderer.rectangle(*position, size)
                    .set_line_width(1.)
                    .set_color(unsafe { &COLOR_THEME.accent_bg_color })
                    .fill_preserve()?
                    .set_color(unsafe { &COLOR_THEME.accent_fg_color })
                    .stroke().map(|_| ())
            }
            Self::Connection {start, position: end , ..} => {
                let offset = Vector2(
                    Vector2(start.0 + ((end.0 - start.0) as f32 * 0.7) as i32, start.1),
                    Vector2(end.0 + ((start.0 - end.0) as f32 * 0.7) as i32, end.1),
                );

                renderer.set_line_width(4.)
                    .set_color(unsafe { &COLOR_THEME.disabled_bg_color })    
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
    fn select_all(&mut self);

    fn unhighlight(&mut self);
    fn selected(&self) -> Vec<uuid::Uuid>;
    fn highlight_area(&mut self);
}
