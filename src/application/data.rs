use std::{
    collections::HashMap,
    io::BufReader,
    fs::File,
    cmp, path::Path
};

use crate::{
    modules::{
        Module,
        builtin
    },
    simulator::Plot
};

use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum Selection {
    Single(u32),
    Many(Vec<u32>),
    Area((i32, i32), (i32, i32)),
    None
}

impl Default for Selection {
    fn default() -> Self {
        Selection::None
    }
}

impl Selection {
    pub fn is_area(&self) -> bool {
        match self {
            Self::Area(_, _) => true,
            _ => false
        }
    }

    pub fn area_start(&self) -> Option<(i32, i32)> {
        match self {
            Self::Area(start, _) => Some(*start),
            _ => None
        }
    }

    pub fn area_end(&self) -> Option<(i32, i32)> {
        match self {
            Self::Area(_, end) => Some(*end),
            _ => None
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ApplicationData {
    modules: HashMap<String, Module>,
    plots: Vec<Plot>,
    current_plot: usize,

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
            plots: vec![Plot::new()],
            current_plot: 0usize,
            selection: Selection::None
        }
    }

    pub fn from_json<P>(path: P) -> Result<Self, String> 
        where P: AsRef<Path>
    {
        let f = File::open(path);
        if let Err(err) = f {
            return Err(err.to_string());
        }

        let reader = BufReader::new(f.unwrap());
    
        let result: serde_json::Result<ApplicationData> = serde_json::from_reader(reader);
        match result {
            Ok(data) => Ok(data),
            Err(err) => Err(err.to_string()),
        }
    }

    pub fn add_module(&mut self, module: Module) -> &mut Self {
        self.modules.insert(module.name().clone(), module);
        self
    }

    pub fn module_exists(&self, name: &String) -> bool {
        self.modules.contains_key(name)
    }

    pub fn get_module(&self, name: &String) -> Option<&Module> {
        match self.modules.get(name) {
            Some(module) => Some(module),
            None => None
        }
    }

    pub fn modules(&self) -> &HashMap<String, Module> {
        &self.modules
    }

    pub fn current_plot(&self) -> &Plot {
        self.plots.get(self.current_plot).unwrap_or_else(|| {
            panic!("Invalid plot `{}` selected, when only `{}` exist. THIS IS A BUG.", self.current_plot, self.plots.len());
        })
    }

    pub fn current_plot_mut(&mut self) -> &mut Plot {
        let len = self.plots.len();
        self.plots.get_mut(self.current_plot).unwrap_or_else(|| {
            panic!("Invalid plot `{}` selected, when only `{}` exist. THIS IS A BUG.", self.current_plot, len);
        })
    }

    pub fn set_selection(&mut self, selection: Selection) {
        //if let Selection::Single(index) = selection {
        //    let last = self.blocks.len() - 1;
        //    self.blocks.swap(index, last);
        //    self.selection = Selection::Single(last);    
        //}
        //else {
            self.selection = selection;
        //}
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    pub fn unhighlight(&mut self) {
        match self.selection.clone() {
            Selection::Single(id) => self.current_plot_mut().get_block_mut(id).unwrap().set_highlighted(false),
            Selection::Many(ids) => ids.iter().for_each(|id| self.current_plot_mut().get_block_mut(*id).unwrap().set_highlighted(false)),
            Selection::Area(_, _) => self.current_plot_mut().blocks_mut().iter_mut().for_each(|(_, v)| v.set_highlighted(false)),
            Selection::None => ()
        }

        self.selection = Selection::None
    }

    pub fn highlight_area(&mut self) {
        if let Selection::Area(selection_start, selection_end) = self.selection {
            let mut selected = Vec::new();

            let x1 = cmp::min(selection_start.0, selection_end.0);
            let y1 = cmp::min(selection_start.1, selection_end.1);
            let x2 = cmp::max(selection_start.0, selection_end.0);
            let y2 = cmp::max(selection_start.1, selection_end.1);
            
            for (_, block) in self.current_plot_mut().blocks_mut().iter_mut() {
                if block.is_in_area((x1, y1, x2, y2)) {
                    block.set_highlighted(true);
                    selected.push(block.id());
                }
            }

            self.selection = Selection::Many(selected)
        }
    }
}