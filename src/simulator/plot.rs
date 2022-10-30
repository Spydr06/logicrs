use super::Block;
use std::collections::HashMap;
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