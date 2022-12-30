pub mod template;
pub mod data;
pub mod selection;

use glib::subclass::types::ObjectSubclassIsExt;
use gtk::{gio, glib::{wrapper, clone}, prelude::ActionMapExt, traits::*, AboutDialog, subclass::prelude::ApplicationImpl};
use crate::config;

wrapper! {
    pub struct Application(ObjectSubclass<template::ApplicationTemplate>)
    @extends gio::Application, gtk::Application, 
    @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl Application {
    pub fn new() -> Self {
        gio::resources_register_include!("logicrs.gresource").expect("Failed to register resources.");
        glib::Object::new(&[
            ("application-id", &"com.spydr06.logicrs"),
            ("flags", &gio::ApplicationFlags::HANDLES_OPEN),
        ]).expect("Failed to create main application struct")
    }

    fn quit(&self) {
        self.imp().shutdown(self);
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let dialog = AboutDialog::builder()
            .transient_for(&window)
            .modal(true)
            .program_name(config::APP_ID)
            .version(config::VERSION)
            .comments(config::DESCRIPTION)
            .copyright(config::COPYRIGHT)
            .authors(config::AUTHORS.split(':').map(|s| s.to_string()).collect())
            .website(config::REPOSITORY)
            .license_type(gtk::License::MitX11)
            .build();
        
        dialog.present();
    }

    pub(self) fn setup_gactions(&self) {
        let quit_action = gio::SimpleAction::new("quit", None);
        quit_action.connect_activate(clone!(@weak self as app => move |_, _| {
            app.quit();
        }));
        self.add_action(&quit_action);

        let about_action = gio::SimpleAction::new("about", None);
        about_action.connect_activate(clone!(@weak self as app => move |_, _| {
            app.show_about();
        }));
        self.add_action(&about_action);

        let save_action = gio::SimpleAction::new("save", None);
        save_action.connect_activate(clone!(@weak self as app => move |_, _| {
            app.imp().save();
        }));
        self.add_action(&save_action);

        let save_as_action = gio::SimpleAction::new("save-as", None);
        save_as_action.connect_activate(clone!(@weak self as app => move |_, _| {
            app.imp().save_as();
        }));
        self.add_action(&save_as_action);
    }
}
