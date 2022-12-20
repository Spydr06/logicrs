mod application;
mod ui;
mod modules;
mod renderer;
mod simulator;
 
#[macro_use]
extern crate log;

use adw::prelude::ApplicationExtManual;
use application::Application;

// std::thread_local! {
//     pub static APPLICATION_DATA: RefCell<ApplicationData> = RefCell::new(ApplicationData::new());
// }

/*fn init_new() {
    APPLICATION_DATA.with(|d| {
        let mut data = d.borrow_mut();

        let and_mod = data.get_module(&"And".to_string()).unwrap();
        let or_mod = data.get_module(&"Or".to_string()).unwrap();
        let not_mod = data.get_module(&"Not".to_string()).unwrap();
        let xor_mod = data.get_module(&"Xor".to_string()).unwrap();

        let mut block1 = Block::new(&and_mod, (10, 10), data.new_id());
        let mut block2 = Block::new(&or_mod, (110, 10), data.new_id());
        let block3 = Block::new(&not_mod, (210, 10), data.new_id());
        let block4 = Block::new(&xor_mod, (310, 10), data.new_id());
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
}*/

fn main() {
    env_logger::init();

    info!("Starting up LogicRs...");
    //init_new();
    
    let application = Application::new();
    std::process::exit(application.run());
}

pub fn die<'a>(reason: &'a str) -> ! {
    error!("{reason}");
    panic!()
}