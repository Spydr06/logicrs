use std::{collections::HashMap, sync::*, fs::{OpenOptions, File}, io::{Write, BufReader}};
use serde::{Serialize, Deserialize};
use gtk::{gio, prelude::FileExt};
use crate::simulator::*;

pub type ProjectRef = Arc<Mutex<Project>>;

#[derive(Serialize, Deserialize)]
pub struct Project {
    modules: HashMap<String, Module>,
    main_plot: Plot,
    id_counter: u32
}

impl Default for Project {
    fn default() -> Self {
        Self::new(&builtin::BUILTINS)
    }
}

impl Project {
    pub fn new(modules: &Vec<Module>) -> Self {
        Self {
            modules: modules.iter().map(|module| (module.name().to_owned(), module.to_owned())).collect::<HashMap<_, _>>(),
            main_plot: Plot::new(),
            id_counter: 0
        }
    }

    pub fn load_from(file: &gio::File) -> Result<Self, String> {
       let f = File::open(file.path().unwrap());
       if let Err(err) = f {
           return Err(err.to_string());
       }
       let reader = BufReader::new(f.unwrap());

       let result: serde_json::Result<Project> = serde_json::from_reader(reader);
       match result {
           Ok(data) => {
               info!("Opened file `{}`", file.path().unwrap().to_str().unwrap());
               Ok(data)
           },
           Err(err) => Err(err.to_string()),
       }
    }

    pub fn write_to(&self, file: &gio::File) -> Result<(), String> {
        info!("Writing to `{}` ...", file.path().unwrap().to_str().unwrap());
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

    pub fn module(&self, name: &String) -> Option<&Module> {
        self.modules.get(name)
    }

    pub fn modules(&self) -> &HashMap<String, Module> {
        &self.modules
    }

    pub fn add_module(&mut self, mut module: Module) {
        let num_inputs = module.get_num_inputs();
        let num_outputs = module.get_num_outputs();

        // generate Input/Output blocks inside the new module
        if let Some(plot) = module.plot_mut() {
            let id = self.new_id();
            let input_module = self.modules.get(&*builtin::INPUT_MODULE_NAME).unwrap();
            plot.add_block(Block::new_sized(&input_module, (50, 50), id, 0,  num_inputs));

            let id = self.new_id();
            let output_module = self.modules.get(&*builtin::OUTPUT_MODULE_NAME).unwrap();
            plot.add_block(Block::new_sized(&output_module, (400, 50), id, num_outputs, 0));
        }

        self.modules.insert(module.name().clone(), module);
    }

    pub fn new_id(&mut self) -> u32 {
        self.id_counter += 1;
        self.id_counter
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
