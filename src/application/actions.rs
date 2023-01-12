use super::*;
use crate::fatal::die;

impl Application {
    pub(super) fn quit(&self) {
        self.close_current_file(glib::clone!(@weak self as app => move |response| {
            match response {
                "Cancel" => return,
                "No" =>  {},
                "Yes" => app.imp().save(),
                _ => panic!("unexpected response \"{}\"", response)
            };

            app.imp().shutdown();
        }));
    }

    pub(super) fn close_current_file<F>(&self, after: F)
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

    pub(super) fn open_new(&self) {
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

    pub(crate) fn open(&self) {
        self.close_current_file(glib::clone!(@weak self as app => move |response| {
            match response {
                "Cancel" => return,
                "No" =>  {},
                "Yes" => app.imp().save(),
                _ => panic!("unexpected response \"{}\"", response)
            };

            let window = app.active_window().unwrap();

            let json_filter = gtk::FileFilter::new();
            json_filter.set_name(Some("JSON files"));
            json_filter.add_mime_type("application/json");

            let open_dialog = gtk::FileChooserNative::builder()
                .transient_for(&window)
                .modal(true)
                .title("Open File")
                .action(gtk::FileChooserAction::Open)
                .accept_label("Open")
                .cancel_label("Cancel")
                .filter(&json_filter)
                .build();
            
            open_dialog.connect_response({
                let obj = app.downgrade();
                let file_chooser = RefCell::new(Some(open_dialog.clone()));
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
                        }
                        else {
                            warn!("got file chooser response more than once");
                        }
                    }
                    else {
                        warn!("got file chooser response after window was freed");
                    }
                }
            });
            
            open_dialog.show();
        }));
    }

    pub(super) fn save_as(&self) {
        //let window = self.active_window().unwrap();

        let json_filter = gtk::FileFilter::new();
        json_filter.set_name(Some("JSON files"));
        json_filter.add_mime_type("application/json");

        let save_dialog = gtk::FileChooserNative::builder()
       //     .transient_for(&window)
            .modal(true)
            .title("Open File")
            .action(gtk::FileChooserAction::Save)
            .accept_label("Save")
            .filter(&json_filter)
            .cancel_label("Cancel")
            .build();

            save_dialog.connect_response({
                let file_chooser = RefCell::new(Some(save_dialog.clone()));
                glib::clone!(@weak self as app => move |_, response| {
                    if let Some(file_chooser) = file_chooser.take() {
                        if response == gtk::ResponseType::Accept {
                            for file in file_chooser.files().snapshot().into_iter() {
                                let file: gio::File = file
                                    .downcast()
                                    .expect("unexpected type returned from file chooser");
                                if !file.query_exists(gio::Cancellable::NONE) {
                                    if let Err(err) = file.create(gio::FileCreateFlags::NONE, gio::Cancellable::NONE) {
                                        die(err.message());
                                    }
                                }
                                app.imp().data().lock().unwrap().set_file(Some(file));
                                if let Err(err) = app.imp().data().lock().unwrap().save() {
                                    die(err.as_str());
                                }
                            }
                        }
                    } else {
                        warn!("got file chooser response more than once");
                    }
                })
            });

        save_dialog.show();
    }

    pub(super) fn show_about(&self) {
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
}