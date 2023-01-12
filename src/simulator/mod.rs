pub mod block;
pub mod connection;
pub mod plot;

pub use {block::*, connection::*, plot::*};
use std::thread::*;
use crate::application::data::ApplicationDataRef;

pub struct Simulator {
    thread: JoinHandle<()>
}

impl Simulator {
    pub fn new(_data: ApplicationDataRef) -> Self {
        Self{
            thread: spawn(|| simulate()),
        }
    }

    pub fn join(self) {
        self.thread.join().unwrap();
    }
}

fn simulate() {
}
