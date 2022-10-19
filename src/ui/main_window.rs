use adw::{self, subclass::prelude::AdwApplicationWindowImpl, ApplicationWindow, Leaflet};

use glib::{
    object_subclass,
    subclass::{
        object::{ObjectImpl, ObjectImplExt},
        types::{ObjectSubclass, ObjectSubclassExt},
        InitializingObject,
    },
    wrapper, Object,
};

use gtk::{
    gio::{ActionGroup, ActionMap},
    prelude::{GObjectPropertyExpressionExt, InitializingWidgetExt},
    subclass::{
        application_window::ApplicationWindowImpl,
        prelude::{WidgetImpl, WindowImpl},
        widget::{CompositeTemplate, WidgetClassSubclassExt},
    },
    Accessible, Buildable, CompositeTemplate, ConstraintTarget, Native, Root, ShortcutManager,
    TemplateChild, Widget, Window, traits::GtkWindowExt,
};

use crate::application::Application;
use super::{
    circuit_panel::{CircuitPanel, CircuitPanelTemplate},
    module_list::{ModuleList, ModuleListTemplate},
};

wrapper! {
    pub struct MainWindow(ObjectSubclass<MainWindowTemplate>)
        @extends gtk::ApplicationWindow, Window, Widget,
        @implements ActionGroup, ActionMap, Accessible, Buildable, ConstraintTarget, Native, Root, ShortcutManager;
}

impl MainWindow {
    pub fn new(app: &Application) -> Self {
        Object::new(&[
                ("application", app),
                ("title", &"LogicRs"),
            ]).expect("failed to create window")
    }
}

#[derive(CompositeTemplate, Default)]
#[template(resource = "/app/main-window.ui")]
pub struct MainWindowTemplate {
    #[template_child]
    pub leaflet: TemplateChild<Leaflet>,

    #[template_child]
    pub module_list: TemplateChild<ModuleList>,

    #[template_child]
    pub circuit_panel: TemplateChild<CircuitPanel>,
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
        obj.set_title(Some("LogicRs"));

        let module_list = self.module_list.get();
        let module_list_template = ModuleListTemplate::from_instance(&module_list);

        let circuit_panel = self.circuit_panel.get();
        let circuit_panel_template = CircuitPanelTemplate::from_instance(&circuit_panel);

        self.leaflet.property_expression("folded").bind(
            &module_list_template.header_bar.get(),
            "show-end-title-buttons",
            Widget::NONE,
        );

        self.leaflet.property_expression("folded").bind(
            &circuit_panel_template.header_bar.get(),
            "show-start-title-buttons",
            Widget::NONE,
        );

        self.leaflet.property_expression("folded").bind(
            &circuit_panel_template.back_button.get(),
            "visible",
            Widget::NONE,
        );
    }
}

impl WidgetImpl for MainWindowTemplate {}
impl WindowImpl for MainWindowTemplate {}
impl ApplicationWindowImpl for MainWindowTemplate {}
impl AdwApplicationWindowImpl for MainWindowTemplate {}
