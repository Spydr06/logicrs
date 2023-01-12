use gtk::{prelude::*, subclass::prelude::*, gio, glib, gdk};
use adw::subclass::prelude::*;
use std::cell::RefCell;
use super::data::*;
use crate::{ui::main_window::MainWindow, simulator::Simulator, fatal::*, modules::Module};

#[derive(Default)]
pub struct ApplicationTemplate {
    data: ApplicationDataRef,
    window: RefCell<Option<MainWindow>>,
    simulator: RefCell<Option<Simulator>>
}

impl ApplicationTemplate {
    const CSS_RESOURCE: &'static str = "/style/style.css";

    fn start_simulation(&self) {
        *self.simulator.borrow_mut() = Some(Simulator::new(self.data.clone()))
    }

    fn stop_simulation(&self) {
        if let Some(simulator) = self.simulator.replace(None) {
            simulator.join();
        }
    }

    fn create_window(&self, application: &super::Application) {
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);

        let provider = gtk::CssProvider::new();
        provider.load_from_resource(Self::CSS_RESOURCE);
        // We give the CssProvided to the default screen so the CSS rules we added
        // can be applied to our window.
        gtk::StyleContext::add_provider_for_display(
            &gdk::Display::default().expect("Could not connect to a display."),
            &provider,
           gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // build the application window and UI
        let window = MainWindow::new(application);
        window.show();
        self.window.replace(Some(window));
    }

    pub fn save(&self) {
        let data = self.data.lock().unwrap();
        if data.file().is_some() { 
            data.save().unwrap_or_die();
        }
    }

    pub fn data(&self) -> ApplicationDataRef {
        self.data.clone()
    }

    pub fn add_module(&self, module: Module) {
        if let Some(window) = self.window.borrow().as_ref() {
            window.add_module_to_ui(&self.instance(), &module);
        }
        self.data.lock().unwrap().add_module(module);
    }

    pub fn delete_selected_blocks(&self) {
        self.data.lock().unwrap().delete_selected();
        if let Some(window) = self.window.borrow().as_ref() {
            window.rerender_circuit();
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for ApplicationTemplate {
    const NAME: &'static str = "Application";
    type Type = super::Application;
    type ParentType = adw::Application;
}

impl ObjectImpl for ApplicationTemplate {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.instance();
        obj.setup_gactions();
        obj.set_accels_for_action("app.quit", &["<primary>Q", "<primary>W"]);
        obj.set_accels_for_action("app.about", &["<primary>comma"]);
        obj.set_accels_for_action("app.save", &["<primary>S"]);
        obj.set_accels_for_action("app.save-as", &["<primary><shift>S"]);
        obj.set_accels_for_action("app.open", &["<primary>O"]);
        obj.set_accels_for_action("app.new", &["<primary>N"]);
        obj.set_accels_for_action("app.delete-block", &["Delete"]);
    }
}
impl ApplicationImpl for ApplicationTemplate {
    fn activate(&self) {
        self.create_window(&self.instance());
        self.start_simulation();
    }

    fn open(&self, files: &[gio::File], _hint: &str) {
        assert!(files.len() != 0);

        let file = &files[0];
        if file.path().is_none() {
            die("File path is None");
        }

        let data = ApplicationData::build(file.to_owned());
        if let Err(err) = data {
            die(err.as_str());
        }

        let mut old_data = self.data.lock().unwrap();
        *old_data = data.unwrap();
        std::mem::drop(old_data);

        self.create_window(&self.instance());
        self.start_simulation();
    }

    fn shutdown(&self) {
        self.stop_simulation();
        self.save();
    }
}
impl GtkApplicationImpl for ApplicationTemplate {}
impl AdwApplicationImpl for ApplicationTemplate {}
impl WidgetImpl for ApplicationTemplate {}
