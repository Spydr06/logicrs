use gtk::{prelude::*, subclass::prelude::*, gio, glib, gdk};
use adw::subclass::prelude::*;
use std::cell::RefCell;
use crate::{
    ui::{main_window::MainWindow, circuit_view::CircuitView, dialogs},
    fatal::*, project::*, simulator::*,
};

use super::action::*;

#[derive(Default)]
pub struct ApplicationTemplate {
    project: ProjectRef,
    window: RefCell<Option<MainWindow>>,
    simulator: RefCell<Option<Simulator>>,
    file: RefCell<Option<gio::File>>,
    action_stack: RefCell<ActionStack>,
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
        if let Some(file) = self.file.borrow().as_ref() { 
            let project = self.project.lock().unwrap();
            project.write_to(file)?;
            if let Some(window) = self.window.borrow().as_ref() {
                window.set_subtitle(&self.file_name());
            }
        }
        else {
            self.instance().save_as();
        }

        self.action_stack().borrow_mut().set_dirty(false);
        Ok(())
    }

    pub fn add_module(&self, module: Module) {
        if let Some(window) = self.window.borrow().as_ref() {
            window.add_module_to_ui(&self.instance(), &module);
        }
        (&mut*self.project.lock().unwrap()).add_module(module);
    }

    pub fn set_project(&self, project: Project, file: Option<gio::File>) {
        let mut old = self.project.lock().unwrap();
        *old = project;

        drop(old);

        self.file.replace(file);
        self.action_stack.borrow_mut().reset();
        if let Some(window) = self.window.borrow().as_ref() {
            window.reset_ui(&self.instance());
        }
    }

    pub fn project(&self) -> &ProjectRef {
        &self.project
    }

    pub fn set_file(&self, file: gio::File) {
        self.file.replace(Some(file));
    }

    pub fn reset(&self) {
        self.set_project(Project::default(), None);
        if let Some(window) = self.window.borrow().as_ref() {
            window.reset_ui(&self.instance());
        }
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

    pub fn current_plot(&self) -> Option<PlotProvider> {
        if let Some(view) = self.current_circuit_view() {
            Some(view.imp().plot_provider())
        }
        else {
            None
        }
    }

    pub fn current_circuit_view(&self) -> Option<CircuitView> {
        self.window.borrow().as_ref()
            .and_then(|window| window.imp().circuit_panel.imp().view.selected_page())
            .and_then(|page| page.child().downcast::<CircuitView>().ok())
    }

    pub fn with_current_plot_mut(&self, func: impl Fn(&mut Plot)) {
        if let Some(view) = self.current_circuit_view() {
            view.imp().plot_provider().with_mut(func);
        }
    }

    pub fn rerender_editor(&self) {
        if let Some(view) = self.current_circuit_view() {
            view.imp().rerender();
        }
    }

    pub fn undo_button(&self) -> gtk::Button {
        self.window.borrow().as_ref().unwrap().panel().undo_button().to_owned()
    }

    pub fn redo_button(&self) -> gtk::Button {
        self.window.borrow().as_ref().unwrap().panel().redo_button().to_owned()
    }

    pub fn action_stack(&self) -> &RefCell<ActionStack> {
        &self.action_stack
    }

    pub fn is_dirty(&self) -> bool {
        self.action_stack.borrow().is_dirty()
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
        obj.set_accels_for_action("app.undo", &["<primary>Z"]);
        obj.set_accels_for_action("app.redo", &["<primary>Y"]);
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

        match Project::load_from(file) {
            Ok(data) => {
                let mut old_data = self.project.lock().unwrap();
                *old_data = data;
                std::mem::drop(old_data);

                self.file.replace(Some(file.to_owned()));
                self.create_window(&self.instance());
                self.start_simulation();
            }
            Err(err) => {
                self.create_window(&self.instance());
                self.start_simulation();

                dialogs::run(self.instance().to_owned(), self.instance().active_window().unwrap(), err, dialogs::basic_error);
            }
        }
    }

    fn shutdown(&self) {
        self.stop_simulation();
        if let Err(err) = self.save() {
            error!("Error saving to: {}: {err}", self.file_name());
        }
    }
}
impl GtkApplicationImpl for ApplicationTemplate {}
impl AdwApplicationImpl for ApplicationTemplate {}
impl WidgetImpl for ApplicationTemplate {}
