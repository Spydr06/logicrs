use std::{
    collections::HashMap,
    sync::Arc,
    cmp
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
    blocks: Vec<Block>,
    
    // selection
    highlighted_block: Option<usize>,
    multiselect_start: Option<(i32, i32)>,
    multiselect_end: Option<(i32, i32)>,
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
            blocks: Vec::new(),
            highlighted_block: None,
            multiselect_end: None,
            multiselect_start: None,
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

    pub fn get_block(&self, index: usize) -> Option<&Block> {
        self.blocks.get(index)
    }

    pub fn get_block_mut(&mut self, index: usize) -> Option<&mut Block> {
        self.blocks.get_mut(index)
    }

    pub fn set_multiselect_start(&mut self, position: Option<(i32, i32)>) {
        self.multiselect_start = position;
    }

    pub fn multiselect_start(&self) -> Option<(i32, i32)> {
        self.multiselect_start
    }

    pub fn set_multiselect_end(&mut self, position: Option<(i32, i32)>) {
        self.multiselect_end = position;
    }

    pub fn multiselect_end(&self) -> Option<(i32, i32)> {
        self.multiselect_end
    }

    pub fn get_block_at(&self, position: (i32, i32)) -> Option<usize> {
        for (i, block) in self.blocks.iter().enumerate() {
            if block.touches(position) {
                return Some(i);
            }
        }

        None
    }

    pub fn get_highlighted_mut(&mut self) -> Option<&mut Block> {
        match self.highlighted_block {
            Some(index) => self.blocks.get_mut(index),
            None => None
        }
    }

    pub fn unhighlight(&mut self) {
        self.blocks.iter_mut().for_each(|v| v.set_highlighted(false));
        self.highlighted_block = None;
    }

    pub fn highlight(&mut self, index: usize) {
        if let Some(old_index) = self.highlighted_block {
            self.blocks.get_mut(old_index).unwrap().set_highlighted(false);
        }

        self.highlighted_block = Some(index);
        self.blocks.get_mut(index).unwrap().set_highlighted(true);
    }

    pub fn highlight_all_selected(&mut self) {
        if self.multiselect_start.is_none() || self.multiselect_end.is_none() {
            return;
        }

        
        let selection_start = self.multiselect_start.unwrap();
        let selection_end = self.multiselect_end.unwrap();

        let x1 = cmp::min(selection_start.0, selection_end.0);
        let y1 = cmp::min(selection_start.1, selection_end.1);
        let x2 = cmp::max(selection_start.0, selection_end.0);
        let y2 = cmp::max(selection_start.1, selection_end.1);
        
        for block in self.blocks.iter_mut() {
            if block.is_in_area((x1, y1, x2, y2)) {
                block.set_highlighted(true);
            }
        }
    }
}