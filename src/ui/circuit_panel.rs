use crate::application::Application;

use super::circuit_view::CircuitView;

use adw::{self, HeaderBar};

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
        prelude::{BoxImpl, WidgetImpl},
        widget::{CompositeTemplate, WidgetClassSubclassExt},
    },
    Accessible, Box, Buildable, Button, CompositeTemplate, ConstraintTarget, Native, Root,
    ShortcutManager, TemplateChild, Widget,
};

wrapper! {
    pub struct CircuitPanel(ObjectSubclass<CircuitPanelTemplate>)
        @extends Box, Widget,
        @implements ActionGroup, ActionMap, Accessible, Buildable, ConstraintTarget, Native, Root, ShortcutManager;
}

impl CircuitPanel {
    pub fn new(app: &Application) -> Self {
        Object::new(&[("application", app)]).expect("failed to create window")
    }
}

#[derive(CompositeTemplate, Default)]
#[template(resource = "/content/circuit-panel.ui")]
pub struct CircuitPanelTemplate {
    #[template_child]
    pub header_bar: TemplateChild<HeaderBar>,

    #[template_child]
    pub back_button: TemplateChild<Button>,

    #[template_child]
    pub circuit_view: TemplateChild<CircuitView>,
}

#[object_subclass]
impl ObjectSubclass for CircuitPanelTemplate {
    const NAME: &'static str = "CircuitPanel";
    type Type = CircuitPanel;
    type ParentType = Box;

    fn class_init(my_class: &mut Self::Class) {
        Self::bind_template(my_class);
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for CircuitPanelTemplate {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj)
    }
}

impl WidgetImpl for CircuitPanelTemplate {}
impl BoxImpl for CircuitPanelTemplate {}
