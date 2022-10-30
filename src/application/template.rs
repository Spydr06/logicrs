use adw::{prelude::WidgetExt, subclass::prelude::AdwApplicationImpl, ColorScheme, StyleManager};
use glib::subclass::{prelude::ObjectImpl, types::ObjectSubclass};
use gtk::{
    subclass::{
        prelude::{ApplicationImpl, GtkApplicationImpl},
        widget::WidgetImpl,
    },
    gdk::Display,
    CssProvider,
    StyleContext,
    STYLE_PROVIDER_PRIORITY_APPLICATION
};

use crate::ui::main_window::MainWindow;
use std::cell::RefCell;

#[derive(Debug, Default)]
pub struct ApplicationTemplate {
    window: RefCell<Option<MainWindow>>,
}

impl ApplicationTemplate {
    const CSS_RESOURCE: &'static str = "/style/style.css";
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

        let provider = CssProvider::new();
        provider.load_from_resource(Self::CSS_RESOURCE);
        // We give the CssProvided to the default screen so the CSS rules we added
        // can be applied to our window.
        StyleContext::add_provider_for_display(
            &Display::default().expect("Could not connect to a display."),
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );


        // build the application window and UI
        let window = MainWindow::new(application);
        window.show();
        *self.window.borrow_mut() = Some(window);
    }
}
impl GtkApplicationImpl for ApplicationTemplate {}
impl AdwApplicationImpl for ApplicationTemplate {}
impl WidgetImpl for ApplicationTemplate {}