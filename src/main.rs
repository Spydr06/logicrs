mod application;
mod ui;
mod modules;
mod renderer;
mod simulator;

use std::cell::RefCell;

use adw::prelude::ApplicationExtManual;
use application::{
    Application,
    data::ApplicationData
};

use simulator::block::Block;

std::thread_local! {
    pub static APPLICATION_DATA: RefCell<ApplicationData> = RefCell::new(ApplicationData::new());
}

fn main() {
    APPLICATION_DATA.with(|data| {
        modules::builtin::register(&mut *data.borrow_mut());
    });

    APPLICATION_DATA.with(|d| {
        let mut data = d.borrow_mut();
        
        let and_mod = data.get_module(&"And".to_string()).unwrap();
        data.add_block(Block::new(and_mod, (10, 10)));

        let or_mod = data.get_module(&"Or".to_string()).unwrap();
        data.add_block(Block::new(or_mod, (110, 10)));

        let not_mod = data.get_module(&"Not".to_string()).unwrap();
        data.add_block(Block::new(not_mod, (210, 10)));

        let xor_mod = data.get_module(&"Xor".to_string()).unwrap();
        data.add_block(Block::new(xor_mod, (310, 10)));
    });

    let application = Application::new();

    std::process::exit(application.run());
}
