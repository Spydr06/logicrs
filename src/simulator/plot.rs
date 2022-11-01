use super::Block;
use crate::renderer::{Renderable, Renderer};
use std::{collections::HashMap, cmp};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Plot {
    blocks: HashMap<u32, Block>   
}

impl Plot {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new()
        }
    }

    pub fn blocks(&self) -> &HashMap<u32, Block> {
        &self.blocks
    }

    pub fn blocks_mut(&mut self) -> &mut HashMap<u32, Block> {
        &mut self.blocks
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.insert(block.id(), block);
    }

    pub fn get_block(&self, id: u32) -> Option<&Block> {
        self.blocks.get(&id)
    }

    pub fn get_block_mut(&mut self, id: u32) -> Option<&mut Block> {
        self.blocks.get_mut(&id)
    }

    pub fn get_block_at(&self, position: (i32, i32)) -> Option<u32> {
        for (i, block) in self.blocks.iter() {
            if block.touches(position) {
                return Some(*i);
            }
        }

        None
    }
}

impl Renderable for Plot {
    fn render<R>(&self, renderer: &R) -> Result<(), R::Error>
        where R: Renderer
    {
        let (width, height) = renderer.size();
        let scale = renderer.scale();

        // render grid
        
        // render all blocks
        renderer.set_line_width(4.);
        for (_, block) in self.blocks() {
            for c in block.connections() {
                if let Some(connection) = c {
                    connection.render(renderer)?;
                }
            }
        }

        renderer.set_line_width(2.);
        for (_, block) in self.blocks() {
            if block.is_in_area((0, 0, (width as f64 / scale) as i32, (height as f64 / scale) as i32)) {
                block.render(renderer)?;
            }
        }

        // render selection
        let selection = crate::APPLICATION_DATA.with(|d| d.borrow().selection().clone());
        if let Some((start_x, start_y)) = selection.area_start().clone() {
            if let Some((end_x, end_y)) = selection.area_end().clone() {
                let position = (cmp::min(start_x, end_x), cmp::min(start_y, end_y));
                let size = (cmp::max(start_x, end_x) - position.0, cmp::max(start_y, end_y) - position.1);

                renderer.rectangle(position, size);
                renderer.set_color(0.2078, 0.5176, 0.894, 0.3);
                renderer.fill_preserve()?;
                renderer.set_color(0.2078, 0.5176, 0.894, 0.7);
                renderer.stroke()?;
            }
        }

        Ok(())
    }
}