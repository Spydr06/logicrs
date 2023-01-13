pub mod block;
pub mod connection;
pub mod plot;
pub mod decoration;
pub mod builtin;
pub mod modules;

pub use {block::*, connection::*, plot::*, decoration::*, modules::*};
use std::thread::*;
use crate::project::ProjectRef;

pub struct Simulator {
    thread: JoinHandle<()>
}

impl Simulator {
    pub fn new(project: ProjectRef) -> Self {
        Self{
            thread: spawn(|| simulate(project)),
        }
    }

    pub fn join(self) {
        self.thread.join().unwrap();
    }
}

fn simulate(_project: ProjectRef) {
}
