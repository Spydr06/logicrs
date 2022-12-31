pub mod template;
pub mod data;
pub mod selection;

use gtk::{prelude::*, subclass::prelude::*, gio, glib};
use crate::config;

glib::wrapper! {
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
        glib::Object::new::<Self>(&[
            ("application-id", &"com.spydr06.logicrs"),
            ("flags", &gio::ApplicationFlags::HANDLES_OPEN),
        ])
    }

    fn quit(&self) {
        self.imp().shutdown();
    }

    fn open_new(&self) {
        
    }

    fn open(&self) {
        
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let dialog = gtk::AboutDialog::builder()
            .transient_for(&window)
            .modal(true)
            .program_name(config::APP_ID)
            .version(config::VERSION)
            .comments(config::DESCRIPTION)
            .copyright(config::COPYRIGHT)
            .authors(config::AUTHORS.split(':').map(|s| s.to_string()).collect())
            .website(config::REPOSITORY)
            .license_type(gtk::License::Gpl30)
            .build();
        
        dialog.present();
    }

    pub(self) fn setup_gactions(&self) {
        let quit_action = gio::SimpleAction::new("quit", None);
        quit_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            app.quit();
        }));
        self.add_action(&quit_action);

        let about_action = gio::SimpleAction::new("about", None);
        about_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            app.show_about();
        }));
        self.add_action(&about_action);

        let save_action = gio::SimpleAction::new("save", None);
        save_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            app.imp().save();
        }));
        self.add_action(&save_action);

        let save_as_action = gio::SimpleAction::new("save-as", None);
        save_as_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            app.imp().save_as();
        }));
        self.add_action(&save_as_action);

        let new_action = gio::SimpleAction::new("new", None);
        new_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            app.open_new();
        }));
        self.add_action(&new_action);

        let open_action = gio::SimpleAction::new("open", None);
        open_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            app.open();
        }));
        self.add_action(&open_action);
    }
}
