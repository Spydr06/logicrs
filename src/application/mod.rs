pub mod template;

use gtk::{gio, glib};

glib::wrapper! {
    pub struct Application(ObjectSubclass<template::ApplicationTemplate>) @extends gio::Application, gtk::Application, @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl Application {
    pub fn new() -> Self {
        gio::resources_register_include!("logicrs.gresource")
            .expect("Failed to register resources.");
        glib::Object::new(&[
            ("application-id", &"org.gtk_rs.application-subclass"),
            ("flags", &gio::ApplicationFlags::empty()),
        ])
        .unwrap()
    }
}
