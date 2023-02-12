use crate::renderer::*;
use serde::{Serialize, Deserialize};

use super::Block;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Decoration {
    None,
    Label(String),
    NotLabel(String),
}

impl Default for Decoration {
    fn default() -> Self {
        Self::None
    }
}

impl Decoration {
    pub(super) fn render<R>(&self, renderer: &R, block: &Block) -> Result<(), R::Error>
        where R: Renderer
    {
        match self {
            Decoration::None => Ok(()),
            Decoration::Label(label) => {
                renderer
                    .set_font_size(26.0)    
                    .move_to((block.position().0 + (block.size().0 / 2 - 7 * label.chars().count() as i32), block.position().1 + (block.size().1 / 2 + 20)))
                    .set_color(unsafe { &COLOR_THEME.decoration_fg_color })
                    .show_text(label)?
                    .set_font_size(DEFAULT_FONT_SIZE);
                Ok(())
            },
            Decoration::NotLabel(label) => {
                let offset = (
                    7 * label.chars().count() as i32,
                    block.position().1 + block.size().1 / 2 - 2
                );
                let position = (
                    block.position().0 + (block.size().0 / 2 - offset.0), 
                    block.position().1 + (block.size().1 / 2 + 20)
                );
                renderer
                    .set_font_size(26.0)    
                    .move_to(position)
                    .set_color(unsafe { &COLOR_THEME.decoration_fg_color })
                    .show_text(label)?
                    .set_font_size(DEFAULT_FONT_SIZE)
                    .move_to((position.0, offset.1))
                    .set_line_width(2.5)
                    .line_to((position.0 + 2 * offset.0, offset.1))
                    .stroke()
                    .map(|_| ())
            }
        }
    }
}
