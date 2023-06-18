pub mod block;
pub mod connection;
pub mod plot;
pub mod decoration;
pub mod builtin;
pub mod modules;
pub mod state;

pub use {block::*, connection::*, plot::*, decoration::*, modules::*, state::*};
use std::{
    thread::{self, JoinHandle},
    time::{Duration, Instant},
    sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc::{Receiver, Sender, self}}, cell::RefCell, collections::{HashMap, HashSet}
};
use gtk::{subclass::prelude::ObjectSubclassIsExt, prelude::Cast};

use crate::{project::{ProjectRef, Project}, ui::{main_window::MainWindow, circuit_view::CircuitView}};

pub trait Identifiable {
    type ID;
}

type UICallbackStore = RefCell<Option<(RefCell<Option<MainWindow>>, Receiver<UICallback>)>>;

thread_local! {
    static UI_CALLBACK: UICallbackStore = RefCell::new(None);
}

pub enum UICallback {
    Redraw,
    Error(String)
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
            Self::Error(err) => {
                if let Some(panel) = window.borrow().as_ref()
                    .map(|window| window.panel()) {
                        panel.push_error(err.clone());
                }              
            }
        }
    }
}

pub type SimResult<T> = Result<T, String>;

pub struct Simulator {
    running: Arc<AtomicBool>,
    thread: JoinHandle<()>
}

impl Simulator {
    pub const DEFAULT_TICKS_PER_SECOND: i32 = 10;

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
        while running.load(Ordering::Relaxed) {
            let start = Instant::now();

            let mut project = project.lock().unwrap();
            let tps = project.tps();
            
            // if we halt the simulation, check again in 0.5 seconds
            if tps == 0 {
                drop(project);
                thread::sleep(Duration::from_millis(500));
                continue;
            }

            let wait_time = Duration::from_secs_f64(1.0 / tps as f64);

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
        let mut call_stack = HashSet::new();
        let mut changes = false;

        project.iter_plots_mut().for_each(|plot| plot.push_state());
        project.iter_plots_mut().for_each(|plot| {
            plot.pop_state();
            match plot.simulate(unsafe { &mut *mut_ref_ptr }, &mut call_stack) {
                Ok(c) => if c { changes = true },
                Err(err) => UICallback::Error(err).handle(tx)
            }
            plot.push_state();
        });
        project.iter_plots_mut().for_each(|plot| plot.pop_state());

        assert!(call_stack.is_empty(), "callstack wasn't empty: {call_stack:?}");
        if changes {
            UICallback::Redraw.handle(tx)
        }
    }
}

pub trait Collect<T, D> {
    fn collect(&self, data: &D) -> T;
}

impl Collect<u128, HashMap<ConnectionID, Connection>> for Vec<Option<ConnectionID>> {
    fn collect(&self, connections: &HashMap<ConnectionID, Connection>) -> u128 {
        let mut inputs = 0;
        for (i, connection_id) in self.iter().enumerate() {
            if let Some(connection) = connection_id.map(|connection_id| connections.get(&connection_id)).flatten() {
                inputs |= (connection.is_active() as u128) << i as u128;
            }
        }
        inputs
    }
}
