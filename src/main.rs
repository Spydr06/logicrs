mod application;
mod ui;
mod modules;
mod renderer;
mod simulator;
mod config;
 
#[macro_use]
extern crate log;

use adw::prelude::ApplicationExtManual;
use application::Application;

fn main() {
    env_logger::init();
    info!("Starting up LogicRs...");    
    
    let application = Application::new();
    std::process::exit(application.run());
}

pub fn die<'a>(reason: &'a str) -> ! {
    error!("{reason}");
    panic!()
}
