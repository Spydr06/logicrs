pub mod info;

use crate::application::info::Info;
use crate::ui;

use adw;
use adw::prelude::*;
use gtk::gio::resources_register_include;

pub struct Application {
    info: Info,
    running: bool,
    adw_app: adw::Application,
}

impl Application {
    pub fn new(info: Info) -> Self {
        resources_register_include!("logicrs.gresource").expect("Failed to register resources.");

        let adw_app = adw::Application::builder()
            .application_id(info.get_app_id())
            .build();

        adw_app.connect_activate(ui::build_ui);

        return Self {
            info,
            running: false,
            adw_app,
        };
    }

    pub fn run(&mut self) -> Result<(), ()> {
        self.running = true;

        self.adw_app.run();

        Ok(())
    }

    pub fn get_adw_app(&self) -> &adw::Application {
        &self.adw_app
    }

    pub fn info(&self) -> &Info {
        &self.info
    }
}
