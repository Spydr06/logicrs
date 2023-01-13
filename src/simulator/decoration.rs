use crate::renderer::*;
use serde::{Serialize, Deserialize};

use super::Block;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Decoration {
    None,
    Label(String),
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
                    .set_color(0.8, 0.8, 0.8, 1.)
                    .show_text(label)?;
                renderer.set_font_size(DEFAULT_FONT_SIZE);
                Ok(())
            }
        }
    }
}