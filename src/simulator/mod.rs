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
    sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc::{Receiver, Sender, self}}, cell::RefCell
};
use gtk::{subclass::prelude::ObjectSubclassIsExt, prelude::Cast};

use crate::{project::{ProjectRef, Project}, ui::{main_window::MainWindow, circuit_view::CircuitView}};

pub trait Identifiable {
    type ID;
}

thread_local! {
    static UI_CALLBACK: RefCell<Option<(RefCell<Option<MainWindow>>, Receiver<UICallback>)>> = RefCell::new(None);
}

pub enum UICallback {
    Redraw
}

impl UICallback {
    pub fn handle(self, tx: &Sender<Self>) {
        tx.send(self).unwrap();

        gtk::glib::source::idle_add_once(|| UI_CALLBACK.with(|ui_callback| {
            if let Some((window, rx)) = &*ui_callback.borrow() {
                let received = rx.recv().unwrap();
                received.exec(window);
            }
        }));
    }

    fn exec(&self, window: &RefCell<Option<MainWindow>>) {
        match self {
            Self::Redraw => {
                if let Some(view) = window.borrow().as_ref()
                    .and_then(|window| window.imp().circuit_panel.imp().view.selected_page())
                    .and_then(|page| page.child().downcast::<CircuitView>().ok()) {
                        view.rerender();
                }
            }
        }
    }
}

pub struct Simulator {
    running: Arc<AtomicBool>,
    thread: JoinHandle<()>
}

impl Simulator {
    const TICKS_PER_SECOND: f64 = 10.0;

    pub fn new(project: ProjectRef, window: RefCell<Option<MainWindow>>) -> Self {
        info!("starting simulation...");

        let (tx, rx) = mpsc::channel();
        UI_CALLBACK.with(|ui_callback| *ui_callback.borrow_mut() = Some((window, rx)));

        let running = Arc::new(AtomicBool::new(true));
        let sim = Self {
            running: running.clone(),
            thread: thread::spawn(move || Self::schedule(running, project, tx)),
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

        UI_CALLBACK.with(|ui_callback| *ui_callback.borrow_mut() = None);

        info!("stopped simulation.");
    }

    fn schedule(running: Arc<AtomicBool>, project: ProjectRef, tx: Sender<UICallback>) {
        let wait_time = Duration::from_secs_f64(1.0 / Self::TICKS_PER_SECOND);
        while running.load(Ordering::Relaxed) {
            let start = Instant::now();

            let mut project = project.lock().unwrap();
            Self::simulate(&mut project, &tx);
            drop(project);

            let runtime = start.elapsed();
            if let Some(remaining) = wait_time.checked_sub(runtime) {
                thread::sleep(remaining);
            }
        }
    }

    fn simulate(project: &mut Project, tx: &Sender<UICallback>) {
        let mut_ref_ptr = project as *mut Project;
        let mut changes = false;
        project.iter_plots_mut().for_each(|plot| {
            if plot.simulate(unsafe { &mut *mut_ref_ptr }) {
                changes = true;
            }
        });

        if changes {
            UICallback::Redraw.handle(tx)
        }
    }
}
