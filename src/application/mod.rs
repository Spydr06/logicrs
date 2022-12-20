pub mod template;
pub mod data;

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
        gio::resources_register_include!("logicrs.gresource").expect("Failed to register resources.");
        let application: Self = glib::Object::new(&[
            ("application-id", &"com.spydr06.logicrs"),
            ("flags", &gio::ApplicationFlags::HANDLES_OPEN),
        ]).unwrap();
        application
    }
}
