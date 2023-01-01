use serde::{Serialize, Deserialize};
use std::{
    collections::HashMap,
    io::{BufReader, Write},
    fs::{File, OpenOptions},
    sync::atomic::{AtomicU32, Ordering},
    cmp,
    sync::{Arc, Mutex}
};
use gtk::gio::{self, prelude::FileExt};
use crate::{
    modules::{
        Module,
        builtin
    },
    simulator::Plot
};
use super::selection::*;

pub type ApplicationDataRef = Arc<Mutex<ApplicationData>>;

#[derive(Serialize, Deserialize)]
pub struct ApplicationData {
    modules: HashMap<String, Module>,
    plots: Vec<Plot>,
    id_counter: AtomicU32,

    #[serde(skip)]
    file: Option<gio::File>,

    #[serde(skip)]
    current_plot: usize,

    #[serde(skip)]
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
        let mut data = Self {
            modules: HashMap::new(),
            plots: vec![Plot::new()],
            current_plot: 0usize,
            id_counter: AtomicU32::new(0u32),
            file: None,
            selection: Selection::None,
        };

        builtin::register(&mut data);
        data
    }

    pub fn reset(&mut self) -> &mut Self {
        self.modules.clear();
        self.plots = vec![Plot::new()];
        self.current_plot = 0usize;
        self.id_counter = AtomicU32::new(0u32);
        self.file = None;
        self.selection = Selection::None;

        builtin::register(self);

        self
    }

    pub fn build(file: gio::File) -> Result<Self, String>  {
        let f = File::open(file.path().unwrap());
        if let Err(err) = f {
            return Err(err.to_string());
        }

        let reader = BufReader::new(f.unwrap());
    
        let result: serde_json::Result<ApplicationData> = serde_json::from_reader(reader);
        match result {
            Ok(mut data) => {
                info!("Opened file `{}`", file.path().unwrap().to_str().unwrap());
                data.set_file(Some(file));
                Ok(data)
            },
            Err(err) => Err(err.to_string()),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        if (&self.file).is_none() {
            return Err("save_as is not implemented".to_string());
        }

        let file = self.file.as_ref().unwrap();

        info!("Saving to `{}` ...", file.path().unwrap().to_str().unwrap());
        let res = OpenOptions::new().write(true).truncate(true).open(file.path().unwrap());
        if let Err(err) = res {
            return Err(err.to_string());
        }

        let mut f = res.unwrap();
        let result = serde_json::to_string(self);
        match result {
            Ok(serialized) => {
                let res = f.write(serialized.as_bytes());
                match res {
                    Ok(bytes_written) => {
                        info!("Wrote {} bytes to `{}` successfully", bytes_written, file.path().unwrap().to_str().unwrap());
                        Ok(())
                    }
                    Err(err) => {
                        Err(err.to_string())
                    }
                }
            },
            Err(err) => Err(err.to_string())
        }
    }

    pub fn filename(&self) -> String {
        match &self.file {
            Some(file) => file.path().unwrap().into_os_string().into_string().unwrap(),
            None => "New File".to_string()
        }
    }

    pub fn new_id(&self) -> u32 {
        self.id_counter.fetch_add(1u32, Ordering::SeqCst)
    }

    pub fn file(&self) -> &Option<gio::File> {
        &self.file
    }

    pub fn set_file(&mut self, file: Option<gio::File>) -> &mut Self {
        self.file = file;
        self
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
            Selection::Connection { block_id: _, output: _, start: _, position: _ } => (),
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