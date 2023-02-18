use std::{collections::HashMap, sync::*, fs::{OpenOptions, File}, io::{Write, BufReader}};
use serde::{Serialize, Deserialize};
use gtk::{gio, prelude::FileExt};
use crate::simulator::*;

pub type ProjectRef = Arc<Mutex<Project>>;

#[derive(Serialize, Deserialize)]
pub struct Project {
    modules: HashMap<String, Module>,
    main_plot: Plot,
}

impl Default for Project {
    fn default() -> Self {
        Self::new(&builtin::BUILTINS)
    }
}


impl Project {
    pub const FILE_EXTENSION: &'static str = "lrsproj";
    pub const FILE_PATTERN: &'static str = "*.lrsproj";

    pub fn file_filter() -> gtk::FileFilter {
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("LogicRs project files"));
        filter.add_pattern(Self::FILE_PATTERN);
        filter.add_pattern(&Self::FILE_PATTERN.to_uppercase());
        filter
    }

    pub fn new(modules: &Vec<Module>) -> Self {
        Self {
            modules: modules.iter().map(|module| (module.name().to_owned(), module.to_owned())).collect(),
            main_plot: Plot::new(),
        }
    }

    pub fn load_from(file: &gio::File) -> Result<Self, String> {
        let f = File::open(file.path().unwrap())
            .map_err(|err| err.to_string())?;
        let project = serde_json::from_reader(BufReader::new(f))
            .map_err(|err| err.to_string())?;
        info!("Opened file `{}`", file.path().unwrap().to_str().unwrap());
        Ok(project)
    }

    pub fn write_to(&self, file: &gio::File) -> Result<(), String> {
        info!("Writing to `{}` ...", file.path().unwrap().to_str().unwrap());
        let mut f = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(file.path().unwrap())
            .map_err(|err| err.to_string())?;

        let serialized = serde_json::to_string(self)
            .map_err(|err| err.to_string())?;
        let bytes_written = f.write(serialized.as_bytes())
            .map_err(|err| err.to_string())?;

        info!("Wrote {bytes_written} bytes to `{}` successfully", file.path().unwrap().to_str().unwrap());
        Ok(())
    }

    pub fn module(&self, name: &String) -> Option<&Module> {
        self.modules.get(name)
    }

    pub fn modules(&self) -> &HashMap<String, Module> {
        &self.modules
    }

    pub fn modules_mut(&mut self) -> &mut HashMap<String, Module> {
        &mut self.modules
    }

    pub fn add_module(&mut self, mut module: Module) {
        let num_inputs = module.get_num_inputs();
        let num_outputs = module.get_num_outputs();

        // generate Input/Output blocks inside the new module
        if let Some(plot) = module.plot_mut() {
            let input_module = self.modules.get(&*builtin::INPUT_MODULE_NAME).unwrap();
            plot.add_block(Block::new_sized(&input_module, (50, 50), true, 0,  num_inputs));

            let output_module = self.modules.get(&*builtin::OUTPUT_MODULE_NAME).unwrap();
            plot.add_block(Block::new_sized(&output_module, (400, 50), true, num_outputs, 0));
        }

        self.modules.insert(module.name().clone(), module);
    }

    pub fn remove_module(&mut self, module_name: &String) {
        self.modules.remove(module_name);
    }

    pub fn main_plot(&self) -> &Plot {
        &self.main_plot
    }

    pub fn plot(&self, module_name: &String) -> Option<&Plot> {
        self.modules.get(module_name).and_then(|module| module.plot().as_ref())
    }

    pub fn main_plot_mut(&mut self) -> &mut Plot {
        &mut self.main_plot
    }

    pub fn plot_mut(&mut self, module_name: &String) -> Option<&mut Plot> {
        self.modules.get_mut(module_name).and_then(|module| module.plot_mut().as_mut())
    }
}
