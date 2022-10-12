use adw::{self, subclass::prelude::AdwApplicationWindowImpl, ApplicationWindow};
use glib::{
    object_subclass,
    subclass::{
        object::{ObjectImpl, ObjectImplExt},
        types::ObjectSubclass,
        InitializingObject,
    },
    wrapper, Object,
};
use gtk::{
    gio::{ActionGroup, ActionMap},
    prelude::InitializingWidgetExt,
    subclass::{
        application_window::ApplicationWindowImpl,
        prelude::{TemplateChild, WidgetImpl, WindowImpl},
        widget::{CompositeTemplate, WidgetClassSubclassExt},
    },
    Accessible, Buildable, Button, CompositeTemplate, ConstraintTarget, Native, Root,
    ShortcutManager, Widget, Window,
};

wrapper! {
    pub struct MainWindow(ObjectSubclass<MainWindowTemplate>)
        @extends gtk::ApplicationWindow, Window, Widget,
        @implements ActionGroup, ActionMap, Accessible, Buildable, ConstraintTarget, Native, Root, ShortcutManager;
}

impl MainWindow {
    pub fn new(app: &adw::Application) -> Self {
        Object::new(&[("application", app)]).expect("failed to create window")
    }
}

#[derive(CompositeTemplate, Default)]
#[template(resource = "/app/main-window.ui")]
pub struct MainWindowTemplate {
    #[template_child]
    pub button: TemplateChild<Button>,
}

#[object_subclass]
impl ObjectSubclass for MainWindowTemplate {
    const NAME: &'static str = "MainWindow";

    type Type = MainWindow;
    type ParentType = ApplicationWindow;

    fn class_init(my_class: &mut Self::Class) {
        Self::bind_template(my_class);
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for MainWindowTemplate {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
    }
}

impl WidgetImpl for MainWindowTemplate {}
impl WindowImpl for MainWindowTemplate {}
impl ApplicationWindowImpl for MainWindowTemplate {}
impl AdwApplicationWindowImpl for MainWindowTemplate {}
