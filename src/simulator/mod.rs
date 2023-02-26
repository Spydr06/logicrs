pub mod block;
pub mod connection;
pub mod plot;
pub mod decoration;
pub mod builtin;
pub mod modules;

pub use {block::*, connection::*, plot::*, decoration::*, modules::*};
use std::{
    thread::{self, JoinHandle},
    time::{Duration, Instant},
    sync::{Arc, atomic::{AtomicBool, Ordering}}
};
use crate::project::{ProjectRef, Project};

pub trait Identifiable {
    type ID;
}

pub struct Simulator {
    running: Arc<AtomicBool>,
    thread: JoinHandle<()>
}

impl Simulator {
    const TICKS_PER_SECOND: f64 = 10.0;

    pub fn new(project: ProjectRef) -> Self {
        info!("starting simulation...");

        let running = Arc::new(AtomicBool::new(true));
        let sim = Self {
            running: running.clone(),
            thread: thread::spawn(move || Self::schedule(running, project)),
        };

        info!("started simulation.");
        sim
    }

    pub fn join(self) {
        info!("stopping simulation...");

        self.running.store(false, Ordering::Relaxed);
        if let Err(err) = self.thread.join() {
            error!("{err:?}");
        }

        info!("stopped simulation.");
    }

    fn schedule(running: Arc<AtomicBool>, project: ProjectRef) {
        let wait_time = Duration::from_secs_f64(1.0 / Self::TICKS_PER_SECOND);
        while running.load(Ordering::Relaxed) {
            let start = Instant::now();

            let mut project = project.lock().unwrap();
            Self::simulate(&mut project);
            drop(project);

            let runtime = start.elapsed();
            if let Some(remaining) = wait_time.checked_sub(runtime) {
                thread::sleep(remaining);
            }
        }
    }

    fn simulate(project: &mut Project) {
        let mut_ref_ptr = project as *mut Project;
        project.iter_plots_mut().for_each(|plot| plot.simulate(unsafe { &mut *mut_ref_ptr }));
    }
}
