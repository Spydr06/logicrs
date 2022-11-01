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
use simulator::{block::Block, Linkage};

std::thread_local! {
    pub static APPLICATION_DATA: RefCell<ApplicationData> = RefCell::new(ApplicationData::new());
}

fn init_new() {
    APPLICATION_DATA.with(|d| {
        let mut data = d.borrow_mut();

        let and_mod = data.get_module(&"And".to_string()).unwrap();
        let or_mod = data.get_module(&"Or".to_string()).unwrap();
        let not_mod = data.get_module(&"Not".to_string()).unwrap();
        let xor_mod = data.get_module(&"Xor".to_string()).unwrap();

        let mut block1 = Block::new(&and_mod, (10, 10));
        let mut block2 = Block::new(&or_mod, (110, 10));
        let block3 = Block::new(&not_mod, (210, 10));
        let block4 = Block::new(&xor_mod, (310, 10));
        block1.connect_to(0u8, Linkage {block_id: 1u32, port: 1u8});
        block2.connect_to(0u8, Linkage {block_id: 2u32, port: 0u8});

        let plot = data.current_plot_mut();
        plot.add_block(block1);
        plot.add_block(block2);
        plot.add_block(block3);
        plot.add_block(block4);
    });

    APPLICATION_DATA.with(|d| {
        println!{
            "{}",
            serde_json::to_string(d.to_owned()).unwrap()
        }
    });
}

fn main() -> std::io::Result<()> {
    let application = Application::new();
    std::process::exit(application.run());
}

pub fn die<'a>(reason: &'a str) -> ! {
    eprintln!("Fatal error: `{reason}`");
    std::process::exit(1)
}