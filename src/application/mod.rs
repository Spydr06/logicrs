pub mod template;
pub mod data;
pub mod selection;

use adw::traits::MessageDialogExt;
use gtk::{prelude::*, subclass::prelude::*, gio, glib};
use crate::{config, application::data::ApplicationData};

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

    fn close_current_file<F>(&self, after: F)
    where
        F: Fn(&str) + 'static,
    {
        let window = self.active_window().unwrap();
        let save_dialog = adw::MessageDialog::builder()
            .transient_for(&window)
            .modal(true)
            .heading("Save File?")
            .body(format!("There are unsaved changes in \"{}\". Do you want to save them?", self.imp().data().lock().unwrap().filename()).as_str())
            .close_response("Cancel")
            .default_response("Yes")
            .build();

        save_dialog.add_response("Yes",  "Yes");
        save_dialog.add_response("Cancel", "Cancel");
        save_dialog.add_response("No", "No");
        save_dialog.set_response_enabled("Yes", true);
        save_dialog.set_response_appearance("Yes", adw::ResponseAppearance::Suggested);
        save_dialog.set_response_appearance("No", adw::ResponseAppearance::Destructive);
        save_dialog.present();

        save_dialog.connect_response(None, move |_, response| after(response));
    }

    fn open_new(&self) {
        self.close_current_file(glib::clone!(@weak self as app => move |response| {
            match response {
                "Cancel" => return,
                "No" =>  {},
                "Yes" => app.imp().save(),
                _ => panic!("unexpected response \"{}\"", response)
            };

            app.imp().data().lock().unwrap().reset();
        }));
    }

    pub fn open(&self) {
        self.close_current_file(glib::clone!(@weak self as app => move |response| {
            match response {
                "Cancel" => return,
                "No" =>  {},
                "Yes" => app.imp().save(),
                _ => panic!("unexpected response \"{}\"", response)
            };

            let window = app.active_window().unwrap();
            let open_dialog = gtk::FileChooserNative::builder()
                .transient_for(&window)
                .modal(true)
                .title("Open File")
                .action(gtk::FileChooserAction::Open)
                .accept_label("Open")
                .cancel_label("Cancel")
                .build();
            
            let json_filter = gtk::FileFilter::new();
            json_filter.set_name(Some("JSON files"));
            json_filter.add_mime_type("application/json");
            open_dialog.add_filter(&json_filter);

            open_dialog.connect_response({
                let obj = app.downgrade();
                let file_chooser = std::cell::RefCell::new(Some(open_dialog.clone()));
                move |_, response| {
                    if let Some(obj) = obj.upgrade() {
                        if let Some(file_chooser) = file_chooser.take() {
                            if response == gtk::ResponseType::Accept {
                                for file in file_chooser.files().snapshot().into_iter() {
                                    let file: gio::File = file
                                        .downcast()
                                        .expect("unexpected type returned from file chooser");

                                    if let Ok(data) = ApplicationData::build(file) {
                                        *obj.imp().data().lock().unwrap() = data;
                                    }
                                    else {
                                        error!("Error opening file");
                                    }
                                }
                            }
                        } else {
                            warn!("got file chooser response more than once");
                        }
                    } else {
                        warn!("got file chooser response after window was freed");
                    }
    
                }
            });
            
            open_dialog.show();
        }));
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let dialog = adw::AboutWindow::builder()
            .transient_for(&window)
            .modal(true)
            .application_name(config::APP_ID)
            .version(config::VERSION)
            .comments(config::DESCRIPTION)
            .copyright(config::COPYRIGHT)
            .developer_name("Spydr06")
            .developers(config::AUTHORS.split(':').map(|s| s.to_string()).collect())
            .website(config::REPOSITORY)
            .issue_url(&(config::REPOSITORY.to_owned() + "/issues"))
            .license_type(gtk::License::MitX11)
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
