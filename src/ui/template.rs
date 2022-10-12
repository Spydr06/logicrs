use adw::{
    self, subclass::prelude::AdwApplicationWindowImpl, ApplicationWindow, HeaderBar, Leaflet,
};

use glib::{
    object_subclass,
    once_cell::sync::Lazy,
    subclass::{
        object::{ObjectImpl, ObjectImplExt},
        types::{ObjectSubclass, ObjectSubclassExt},
        InitializingObject,
    },
    ParamFlags, ParamSpec, ParamSpecBoolean, ToValue, Value,
};

use gtk::{
    prelude::{GObjectPropertyExpressionExt, InitializingWidgetExt},
    subclass::{
        application_window::ApplicationWindowImpl,
        prelude::{BoxImpl, WidgetImpl, WindowImpl},
        widget::{CompositeTemplate, WidgetClassSubclassExt},
    },
    Box, Button, CompositeTemplate, TemplateChild, Widget,
};

use super::{circuit_view::CircuitView, main_window::MainWindow, module_list::ModuleList};

#[derive(CompositeTemplate, Default)]
#[template(resource = "/app/circuit-view.ui")]
pub struct CircuitViewTemplate {
    #[template_child]
    pub header_bar: TemplateChild<HeaderBar>,

    #[template_child]
    pub back_button: TemplateChild<Button>,
}

#[object_subclass]
impl ObjectSubclass for CircuitViewTemplate {
    const NAME: &'static str = "CircuitView";
    type Type = CircuitView;
    type ParentType = Box;

    fn class_init(my_class: &mut Self::Class) {
        Self::bind_template(my_class);
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for CircuitViewTemplate {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj)
    }
}

impl WidgetImpl for CircuitViewTemplate {}
impl BoxImpl for CircuitViewTemplate {}

#[derive(CompositeTemplate, Default)]
#[template(resource = "/app/module-list.ui")]
pub struct ModuleListTemplate {
    #[template_child]
    pub header_bar: TemplateChild<HeaderBar>,
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
        self.parent_constructed(obj)
    }
}

impl WidgetImpl for ModuleListTemplate {}
impl BoxImpl for ModuleListTemplate {}

#[derive(CompositeTemplate, Default)]
#[template(resource = "/app/main-window.ui")]
pub struct MainWindowTemplate {
    #[template_child]
    pub leaflet: TemplateChild<Leaflet>,

    #[template_child]
    pub module_list: TemplateChild<ModuleList>,

    #[template_child]
    pub circuit_view: TemplateChild<CircuitView>,
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

        let module_list = self.module_list.get();
        let module_list_template = ModuleListTemplate::from_instance(&module_list);

        let circuit_view = self.circuit_view.get();
        let circuit_view_template = CircuitViewTemplate::from_instance(&circuit_view);

        self.leaflet.property_expression("folded").bind(
            &module_list_template.header_bar.get(),
            "show-end-title-buttons",
            Widget::NONE,
        );

        self.leaflet.property_expression("folded").bind(
            &circuit_view_template.header_bar.get(),
            "show-start-title-buttons",
            Widget::NONE,
        );

        self.leaflet.property_expression("folded").bind(
            &circuit_view_template.back_button.get(),
            "visible",
            Widget::NONE,
        );
    }
}

impl WidgetImpl for MainWindowTemplate {}
impl WindowImpl for MainWindowTemplate {}
impl ApplicationWindowImpl for MainWindowTemplate {}
impl AdwApplicationWindowImpl for MainWindowTemplate {}
