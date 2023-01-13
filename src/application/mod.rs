pub mod template;
pub mod actions;

use std::cell::RefCell;
use adw::traits::MessageDialogExt;
use gtk::{prelude::*, subclass::prelude::*, gio, glib};
use crate::{config, ui::dialogs};

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
            if let Err(err) = app.imp().save() {
                let message =  format!("Error saving to '{}': {}", app.imp().file_name(), err);
                error!("{}", message);
                if let Some(window) = app.active_window() {
                    dialogs::run(app, window, message, dialogs::basic_error);
                }
            }
        }));
        self.add_action(&save_action);

        let save_as_action = gio::SimpleAction::new("save-as", None);
        save_as_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            app.save_as();
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

        let delete_block_action = gio::SimpleAction::new("delete-block", None);
        delete_block_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            app.imp().delete_selected_blocks();
        }));
        self.add_action(&delete_block_action);

        let create_new_module_action = gio::SimpleAction::new("create-new-module", None);
        create_new_module_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            if let Some(window) = app.active_window() {
                dialogs::run(app, window, (), dialogs::new_module); 
            }
        }));
        self.add_action(&create_new_module_action);
    }
}
