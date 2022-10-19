use std::{
    collections::{
        HashMap,
        hash_map::Values
    },
    sync::Arc
};

use crate::{
    modules::{
        Module,
        builtin
    },
    simulator::block::Block
};

pub struct ApplicationData {
    modules: HashMap<String, Arc<Module>>,
    blocks: Vec<Block>
}

impl Default for ApplicationData {
    fn default() -> Self {
        let mut data = Self::new();
        builtin::register(&mut data);

        data
    }
}

impl ApplicationData {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            blocks: Vec::new()
        }
    }

    pub fn add_module(&mut self, module: Module) -> &mut Self
    {
        self.modules.insert(module.get_name().clone(), Arc::new(module));
        self
    }

    pub fn module_exists(&self, name: &String) -> bool {
        self.modules.contains_key(name)
    }

    pub fn get_module(&self, name: &String) -> Option<Arc<Module>> {
        match self.modules.get(name) {
            Some(module) => Some(module.clone()),
            None => None
        }
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn get_blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
}