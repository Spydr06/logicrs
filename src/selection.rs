use serde::{Serialize, Deserialize};

use crate::{renderer::Renderable, simulator::Plot};
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

impl Default for Selection {
    fn default() -> Self {
        Selection::None
    }
}

impl Renderable for Selection {
    fn render<R>(&self, renderer: &R, _data: &Plot) -> Result<(), R::Error>
        where R: crate::renderer::Renderer {

        match self {
            Selection::Area(start, end) => {
                let position = (cmp::min(start.0, end.0), cmp::min(start.1, end.1));
                let size = (cmp::max(start.0, end.0) - position.0, cmp::max(start.1, end.1) - position.1);
                renderer.rectangle(position, size)
                    .set_color(0.2078, 0.5176, 0.894, 0.3)
                    .fill_preserve()?;
                renderer.set_color(0.2078, 0.5176, 0.894, 0.7)
                    .stroke()
            }
            Selection::Connection { block_id: _, output: _, start, position: end } => {
                let offset = (
                    (start.0 + ((end.0 - start.0) as f32 * 0.7) as i32, start.1),
                    (end.0 + ((start.0 - end.0) as f32 * 0.7) as i32, end.1),
                );

                renderer.set_line_width(4.)
                    .set_color(0.346, 0.155, 0.41, 1.)    
                    .move_to(*start)
                    .curve_to(offset.0, offset.1, *end)
                    .stroke()
            }
            _ => Ok(())
        }
    }
}

pub trait SelectionField {
    fn selection(&self) -> &Selection;
    fn set_selection(&mut self, selection: Selection);

    fn unhighlight(&mut self);
    fn delete_selected(&mut self);
    fn highlight_area(&mut self);
}
