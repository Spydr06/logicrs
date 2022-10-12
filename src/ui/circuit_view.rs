use adw;
use glib::{wrapper, Object};
use gtk::{
    gio::{ActionGroup, ActionMap},
    Accessible, Buildable, ConstraintTarget, Native, Root, ShortcutManager, Widget, Window,
};

use super::template::*;

wrapper! {
    pub struct CircuitView(ObjectSubclass<CircuitViewTemplate>)
        @extends gtk::ApplicationWindow, Window, Widget,
        @implements ActionGroup, ActionMap, Accessible, Buildable, ConstraintTarget, Native, Root, ShortcutManager;
}

impl CircuitView {
    pub fn new(app: &adw::Application) -> Self {
        Object::new(&[("application", app)]).expect("failed to create window")
    }
}
