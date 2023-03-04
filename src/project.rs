use std::{collections::*, sync::*, fs::{OpenOptions, File}, io::{Write, BufReader}};
use serde::{Serialize, Deserialize, ser::SerializeStruct};
use gtk::{gio, prelude::FileExt};
use crate::{simulator::{*, builtin::BUILTINS}, renderer::vector::Vector2};

pub type ProjectRef = Arc<Mutex<Project>>;

#[derive(Deserialize)]
pub struct Project {
    modules: HashMap<String, Module>,
    main_plot: Plot
}

impl Default for Project {
    fn default() -> Self {
        Self::new(builtin::BUILTINS.iter().map(|(_, builtin)| builtin.module().clone()).collect())
    }
}

impl Serialize for Project {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        let mut state = serializer.serialize_struct("Project", 2)?;
        state.serialize_field("modules", &HashMap::<&String, &Module>::from_iter(self.modules.iter().filter(|(_, module)| !module.builtin())))?;
        state.serialize_field("main_plot", &self.main_plot)?;
        state.end()
    }
}

impl Project {
    pub const FILE_EXTENSION: &'static str = "lrsproj";
    pub const FILE_PATTERN: &'static str = "*.lrsproj";

    pub fn file_filter() -> gtk::FileFilter {
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("LogicRs project files"));
        filter.add_pattern(Self::FILE_PATTERN);
        filter
    }

    pub fn new(modules: Vec<Module>) -> Self {
        Self {
            modules: modules.iter().map(|module| (module.name().to_owned(), module.clone())).collect(),
            main_plot: Plot::new(),
        }
    }

    pub fn load_from(file: &gio::File) -> Result<Self, String> {
        let f = File::open(file.path().unwrap())
            .map_err(|err| err.to_string())?;
        let mut project: Self = serde_json::from_reader(BufReader::new(f))
            .map_err(|err| err.to_string())?;

        BUILTINS.iter().for_each(|(_, builtin)| project.add_module(builtin.module().clone()));

        info!("Loaded from file `{}`", file.path().unwrap().to_str().unwrap());
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

    pub fn module_mut(&mut self, name: &String) -> Option<&mut Module> {
        self.modules.get_mut(name)
    }

    pub fn modules(&self) -> &HashMap<String, Module> {
        &self.modules
    }

    pub fn modules_mut(&mut self) -> &mut HashMap<String, Module> {
        &mut self.modules
    }

    pub fn add_module(&mut self, mut module: Module) {
        if module.plot().is_some() {
            let num_inputs = module.get_num_inputs();
            let num_outputs = module.get_num_outputs();
            
            let input_module = self.modules.get(&*builtin::INPUT_MODULE_NAME).unwrap();
            let input_block = Block::new_sized(&input_module, Vector2(50, 50), true, 0,  num_inputs);
            
            let output_module = self.modules.get(&*builtin::OUTPUT_MODULE_NAME).unwrap();
            let output_block = Block::new_sized(&output_module, Vector2(400, 50), true, num_outputs, 0);

            module.set_io_blocks(input_block.id(), output_block.id());

            // generate Input/Output blocks inside the new module
            let plot = module.plot_mut().unwrap();
            plot.add_block(input_block);
            plot.add_block(output_block);
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
        self.modules.get(module_name).and_then(|module| module.plot())
    }

    pub fn main_plot_mut(&mut self) -> &mut Plot {
        &mut self.main_plot
    }

    pub fn plot_mut(&mut self, module_name: &String) -> Option<&mut Plot> {
        self.modules.get_mut(module_name).and_then(|module| module.plot_mut())
    }

    pub fn iter_plots_mut(&mut self) -> impl Iterator<Item = &mut Plot> {
        self.modules.iter_mut()
            .filter_map(|(_, module)| module.plot_mut())
            .chain(std::iter::once(&mut self.main_plot))
    }
}
