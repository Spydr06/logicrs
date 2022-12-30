use crate::application::{Application, data::ApplicationDataRef};
use super::circuit_view::CircuitView;
use adw::{self, HeaderBar, TabPage, TabView, TabBar, WindowTitle};

use glib::{
    object_subclass,
    subclass::{
        object::{ObjectImpl, ObjectImplExt},
        types::{ObjectSubclass, ObjectSubclassIsExt},
        InitializingObject,
    },
    wrapper, Object, Cast,
};

use gtk::{
    gio::*,
    prelude::{InitializingWidgetExt},
    subclass::{
        prelude::{BoxImpl, WidgetImpl},
        widget::{CompositeTemplate, WidgetClassSubclassExt},
    },
    Accessible, Box, Buildable, Button, CompositeTemplate, ConstraintTarget, Native, Root,
    ShortcutManager, TemplateChild, Widget,
};

use std::cell::RefCell;

wrapper! {
    pub struct CircuitPanel(ObjectSubclass<CircuitPanelTemplate>)
        @extends Box, Widget,
        @implements ActionGroup, ActionMap, Accessible, Buildable, ConstraintTarget, Native, Root, ShortcutManager;
}

impl CircuitPanel {
    pub fn new(app: &Application, data: ApplicationDataRef) -> Self {
        let panel: Self = Object::new(&[("application", app)]).expect("failed to create window");
        panel.imp().set_title(data.lock().unwrap().filename().as_str());
        panel.imp().set_data(data);
        panel
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
    pub view: TemplateChild<TabView>,

    #[template_child]
    pub tab_bar: TemplateChild<TabBar>,

    data: RefCell<ApplicationDataRef>,
}

impl CircuitPanelTemplate {
    fn add_page<'a>(&self, content: &CircuitView, title: &'a str) -> TabPage {
        let page = self.view.add_page(content, None);
        page.set_indicator_activatable(true);
        page.set_title(title);
        page
    }

    pub fn set_data(&self, data: ApplicationDataRef) {
        self.data.replace(data);
    }

    pub fn new_tab<'a>(&self, title: &'a str) -> TabPage {
        let content = CircuitView::new(self.data.borrow().clone());
        let page = self.add_page(&content, title);
        self.view.set_selected_page(&page);

        page
    }

    pub fn set_title<'a>(&self, title: &'a str) {
        (self.header_bar.title_widget().unwrap().downcast_ref().unwrap() as &WindowTitle).set_subtitle(title);
    }
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
        self.parent_constructed(obj);
       // self.new_tab("Main");
       // self.new_tab("Second");
    }
}

impl WidgetImpl for CircuitPanelTemplate {}
impl BoxImpl for CircuitPanelTemplate {}
