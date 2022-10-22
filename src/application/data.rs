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

#[derive(Clone, Copy)]
pub enum Selection {
    Single(usize),
    Area(Option<(i32, i32)>, Option<(i32, i32)>),
    None
}

impl Selection {
    pub fn is_area(self) -> bool {
        match self {
            Self::Area(_, _) => true,
            _ => false
        }
    }

    pub fn area_start(self) -> Option<(i32, i32)> {
        match self {
            Self::Area(start, _) => start,
            _ => None
        }
    }

    pub fn area_end(self) -> Option<(i32, i32)> {
        match self {
            Self::Area(_, end) => end,
            _ => None
        }
    }
}

pub struct ApplicationData {
    modules: HashMap<String, Arc<Module>>,
    blocks: Vec<Block>,

    selection: Selection
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
            selection: Selection::None
        }
    }

    pub fn add_module(&mut self, module: Module) -> &mut Self {
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

    pub fn modules(&self) -> &HashMap<String, Arc<Module>> {
        &self.modules
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

    pub fn set_selection(&mut self, selection: Selection) {
        if let Selection::Single(index) = selection {
            let last = self.blocks.len() - 1;
            self.blocks.swap(index, last);
            self.selection = Selection::Single(last);    
        }
        else {
            self.selection = selection;
        }
    }

    pub fn selection(&self) -> Selection {
        self.selection
    }

    pub fn get_block_at(&self, position: (i32, i32)) -> Option<usize> {
        for (i, block) in self.blocks.iter().enumerate().rev() {
            if block.touches(position) {
                return Some(i);
            }
        }

        None
    }

    pub fn unhighlight(&mut self) {
        self.blocks.iter_mut().for_each(|v| v.set_highlighted(false));
        self.selection = Selection::None
    }

    pub fn highlight_area(&mut self) {
        if let Selection::Area(selection_start, selection_end) = self.selection {
            if selection_start.is_none() || selection_end.is_none() {
                return;
            }

            let selection_start = selection_start.unwrap();
            let selection_end = selection_end.unwrap();

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
}