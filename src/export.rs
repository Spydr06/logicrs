use crate::{simulator::Module, project::Project, FileExtension, application::Application};

use serde::{Serialize, Deserialize};
use gtk::{gio, prelude::FileExt, subclass::prelude::ObjectSubclassIsExt};
use std::{fs::{OpenOptions, File}, io::{Write, BufReader}, collections::HashMap};

#[derive(Serialize, Deserialize)]
pub struct ModuleFile {
    main_name: String,
    modules: HashMap<String, Module>
}

impl FileExtension for ModuleFile {
    const FILE_EXTENSION: &'static str = "lrsmod";
    const FILE_PATTERN: &'static str = "*.lrsmod";

    fn file_filter() -> gtk::FileFilter {
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("LogicRs module files"));
        filter.add_pattern(Self::FILE_PATTERN);
        filter
    }
}

impl ModuleFile {
    pub fn from_existing(project: &Project, mod_name: String) -> Option<Self> {
        project.module(&mod_name).map(|module| {
            let mut mod_file = Self {
                main_name: mod_name.clone(),
                modules: HashMap::from([(mod_name.clone(), module.clone())])
            };

            project.collect_dependencies(&mod_name, &mut mod_file.modules);
            mod_file
        })
    }

    pub fn export(&self, file: &gio::File) -> Result<(), String> {
        info!("Exporting to `{}`...", file.path().unwrap().to_str().unwrap());
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

    pub fn import(file: &gio::File) -> Result<Self, String> {
        let f = File::open(file.path().unwrap())
            .map_err(|err| err.to_string())?;
        let mod_file: Self = serde_json::from_reader(BufReader::new(f))
            .map_err(|err| err.to_string())?;

        info!("Imported module `{}` from file `{}`", mod_file.main_name, file.path().unwrap().to_str().unwrap());
        Ok(mod_file)
    }

    fn check_compat(&self, project: &mut Project) -> Vec<String> {
        self.modules.keys()
            .cloned()
            .filter(|name| project.module(name).is_some())
            .collect::<Vec<_>>()
    }

    pub fn merge(self, app: &Application) -> Result<(), String> {
        let project = &mut app.imp().project().lock().unwrap();
        let window = app.imp().window().borrow();
        let window = window.as_ref().unwrap();

        let conflicting = self.check_compat(project);
        if conflicting.is_empty() {
            // everything's good
            for (_, module) in self.modules.into_iter() {
                window.add_module_to_ui(app, &module);
                project.add_existing_module(module);
            }
            return Ok(());
        }

        // construct error message
        let message = format!(
            "Error importing `{}`; Conflicting modules exist:\n\t{}",
            self.main_name, conflicting.join(",\n\t")
        );
        Err(message)
    }
}
