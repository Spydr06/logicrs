use super::Block;
use crate::{renderer::{Renderable, Renderer}, application::data::ApplicationData};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
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
    fn render<R>(&self, renderer: &R, data: &ApplicationData) -> Result<(), R::Error>
        where R: Renderer
    {
        let (width, height) = renderer.size();
        let scale = renderer.scale();

        // render grid
        renderer.set_line_width(4.);
        for (_, block) in self.blocks() {
            for c in block.connections() {
                if let Some(connection) = c {
                    connection.render(renderer, data)?;
                }
            }
        }
        
        // render all blocks
        renderer.set_line_width(2.);
        for (_, block) in self.blocks() {
            if block.is_in_area((0, 0, (width as f64 / scale) as i32, (height as f64 / scale) as i32)) {
                block.render(renderer, data)?;
            }
        }

        Ok(())
    }
}