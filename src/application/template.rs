use gtk::{prelude::*, subclass::prelude::*, gio, glib, gdk};
use adw::subclass::prelude::*;
use std::cell::RefCell;
use crate::{
    ui::{main_window::MainWindow, circuit_view::CircuitView},
    fatal::*, modules::*, project::*, selection::*,
    simulator::*,
};

#[derive(Default)]
pub struct ApplicationTemplate {
    project: ProjectRef,
    window: RefCell<Option<MainWindow>>,
    simulator: RefCell<Option<Simulator>>,
    file: RefCell<Option<gio::File>>,
}

impl ApplicationTemplate {
    const CSS_RESOURCE: &'static str = "/style/style.css";

    fn start_simulation(&self) {
        *self.simulator.borrow_mut() = Some(Simulator::new(self.project.clone()))
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

    pub fn save(&self) -> Result<(), String> {
        let project = self.project.lock().unwrap();
        if let Some(file) = self.file.borrow().as_ref() { 
            project.write_to(file)
        }
        else {
            Err(String::from("File was none"))
        }
    }

    pub fn add_module(&self, module: Module) {
        if let Some(window) = self.window.borrow().as_ref() {
            window.add_module_to_ui(&self.instance(), &module);
        }
        (&mut*self.project.lock().unwrap()).add_module(module);
    }

    pub fn delete_selected_blocks(&self) {
        self.with_current_plot_mut(|plot| {
                plot.delete_selected();
                self.window.borrow().as_ref().unwrap().rerender_circuit();
        });
    }

    pub fn set_project(&self, project: Project) {
        let mut old = self.project.lock().unwrap();
        *old = project;
    }

    pub fn project(&self) -> &ProjectRef {
        &self.project
    }

    pub fn set_file(&self, file: gio::File) {
        self.file.replace(Some(file));
    }

    pub fn reset(&self) {
        self.set_project(Project::default());
        self.file.replace(None);
    }

    pub fn file_name(&self) -> String {
        match self.file.borrow().as_ref() {
            Some(file) => file.path().unwrap().into_os_string().into_string().unwrap(),
            None => String::from("New File")
        }
    }

    // pub fn with_current_plot(&self, func: impl Fn(&Plot)) {
    //     if let Some(window) = self.window.borrow().as_ref() {
    //         if let Some(page) = window.imp().circuit_panel.imp().view.selected_page() {
    //             if let Ok(view) = page.child().downcast::<CircuitView>() {
    //                 view.imp().plot_provider().with(func);
    //             }
    //         }
    //     }
    // }

    pub fn with_current_plot_mut(&self, func: impl Fn(&mut Plot)) {
        if let Some(window) = self.window.borrow().as_ref() {
            if let Some(page) = window.imp().circuit_panel.imp().view.selected_page() {
                if let Ok(view) = page.child().downcast::<CircuitView>() {
                    view.imp().plot_provider().with_mut(func);
                }
            }
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

        let data = Project::load_from(file);
        if let Err(err) = data {
            die(err.as_str());
        }
        
        let mut old_data = self.project.lock().unwrap();
        *old_data = data.unwrap();
        std::mem::drop(old_data);
        
        self.file.replace(Some(file.to_owned()));
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
