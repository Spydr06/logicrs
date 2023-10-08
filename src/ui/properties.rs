use gtk::{gio, glib, prelude::*, subclass::prelude::*};

glib::wrapper! {
    pub struct Properties(ObjectSubclass<PropertiesTemplate>)
        @extends gtk::Box, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Properties {}

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/content/properties.ui")]
pub struct PropertiesTemplate {}

impl PropertiesTemplate {}

#[glib::object_subclass]
impl ObjectSubclass for PropertiesTemplate {
    const NAME: &'static str = "Properties";
    type Type = Properties;
    type ParentType = gtk::Box;

    fn class_init(class: &mut Self::Class) {
        Self::bind_template(class);
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for PropertiesTemplate {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

impl WidgetImpl for PropertiesTemplate {
    fn realize(&self) {
        self.parent_realize();
    }

    fn unrealize(&self) {
        self.parent_unrealize();
    }
}

impl BoxImpl for PropertiesTemplate {}
