use adw::{prelude::WidgetExt, subclass::prelude::AdwApplicationImpl, ColorScheme, StyleManager};
use glib::subclass::{prelude::ObjectImpl, types::ObjectSubclass};
use gtk::subclass::{
    prelude::{ApplicationImpl, GtkApplicationImpl},
    widget::WidgetImpl,
};

use crate::ui::main_window::MainWindow;
use std::cell::RefCell;

#[derive(Debug, Default)]
pub struct ApplicationTemplate {
    window: RefCell<Option<MainWindow>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ApplicationTemplate {
    const NAME: &'static str = "Application";
    type Type = super::Application;
    type ParentType = adw::Application;
}

impl ObjectImpl for ApplicationTemplate {}
impl ApplicationImpl for ApplicationTemplate {
    fn activate(&self, application: &Self::Type) {
        StyleManager::default().set_color_scheme(ColorScheme::ForceDark);

        self.window.replace({
            let window = MainWindow::new(application);
            window.show();
            Some(window)
        });
    }
}
impl GtkApplicationImpl for ApplicationTemplate {}
impl AdwApplicationImpl for ApplicationTemplate {}
impl WidgetImpl for ApplicationTemplate {}
