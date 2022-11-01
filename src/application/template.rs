use adw::{prelude::WidgetExt, subclass::prelude::AdwApplicationImpl, ColorScheme, StyleManager};
use glib::subclass::{prelude::ObjectImpl, types::ObjectSubclass};
use gtk::{
    subclass::{
        prelude::{ApplicationImpl, GtkApplicationImpl},
        widget::WidgetImpl,
    },
    gio::{File, prelude::FileExt},
    gdk::Display,
    CssProvider,
    StyleContext,
    STYLE_PROVIDER_PRIORITY_APPLICATION
};
use super::data::ApplicationData;
use crate::ui::main_window::MainWindow;

#[derive(Debug, Default)]
pub struct ApplicationTemplate {
}

impl ApplicationTemplate {
    const CSS_RESOURCE: &'static str = "/style/style.css";

    fn create_window(&self, application: &super::Application) {
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
    }
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
        self.create_window(application);
    }

    fn open(&self, application: &Self::Type, files: &[File], _hint: &str) {
        assert!(files.len() != 0);

        let file = &files[0];
        if file.path().is_none() {
            crate::die("File path is None");
        }

        let data = ApplicationData::build(file.path().unwrap());
        if let Err(err) = data {
            crate::die(err.as_str());
        }

        crate::APPLICATION_DATA.with(|d| d.replace(data.unwrap()));
        self.create_window(application);
    }
}
impl GtkApplicationImpl for ApplicationTemplate {}
impl AdwApplicationImpl for ApplicationTemplate {}
impl WidgetImpl for ApplicationTemplate {}
