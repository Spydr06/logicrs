use adw::{self, HeaderBar};

use glib::{
    object_subclass,
    subclass::{
        object::{ObjectImpl, ObjectImplExt},
        types::ObjectSubclass,
        InitializingObject,
    },
    wrapper,
};

use gtk::{
    prelude::InitializingWidgetExt,
    subclass::{
        prelude::{BoxImpl, WidgetImpl},
        widget::{CompositeTemplate, WidgetClassSubclassExt},
    },
    Accessible, Box, Buildable, Button, CompositeTemplate, ConstraintTarget, Orientable,
    TemplateChild, Widget,
};

use crate::application::Application;
use super::dialogs;

wrapper! {
    pub struct ModuleList(ObjectSubclass<ModuleListTemplate>)
        @extends Widget, Box,
        @implements Accessible, Buildable, ConstraintTarget, Orientable;
}

impl ModuleList {
    pub fn new(app: &Application) -> Self {
        glib::Object::new(&[("application", app)]).expect("Failed to create an instance of ModuleList")
    }
}

#[derive(CompositeTemplate, Default)]
#[template(resource = "/app/module-list.ui")]
pub struct ModuleListTemplate {
    #[template_child]
    pub header_bar: TemplateChild<HeaderBar>,

    #[template_child]
    pub new_module_button: TemplateChild<Button>,
}

#[object_subclass]
impl ObjectSubclass for ModuleListTemplate {
    const NAME: &'static str = "ModuleList";
    type Type = ModuleList;
    type ParentType = Box;

    fn class_init(my_class: &mut Self::Class) {
        Self::bind_template(my_class);
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for ModuleListTemplate {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        dialogs::new(&self.new_module_button, (400, 70), dialogs::new_module)
    }
}

impl WidgetImpl for ModuleListTemplate {}
impl BoxImpl for ModuleListTemplate {}
