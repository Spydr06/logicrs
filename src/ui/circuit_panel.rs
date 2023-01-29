use crate::{application::Application, simulator::PlotProvider};
use super::circuit_view::CircuitView;
use gtk::{prelude::*, subclass::prelude::*, gio, glib};

use std::{cell::RefCell, collections::HashMap};

glib::wrapper! {
    pub struct CircuitPanel(ObjectSubclass<CircuitPanelTemplate>)
        @extends gtk::Box, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

#[gtk::template_callbacks]
impl CircuitPanel {
    pub fn new(app: Application) -> Self {
        let panel: Self = glib::Object::new::<Self>(&[]);
        panel.imp().set_title(app.imp().file_name().as_str());
        panel.imp().set_application(app);
        panel
    }

    #[template_callback]
    pub fn undo_latest(&self, _btn: &gtk::Button) {
        self.imp().application.borrow().undo_action();
    }

    #[template_callback]
    pub fn redo_latest(&self, _btn: &gtk::Button) {
        self.imp().application.borrow().redo_action();
    }

    pub fn undo_button(&self) -> &gtk::Button {
        &self.imp().undo_button
    }

    pub fn redo_button(&self) -> &gtk::Button {
        &self.imp().redo_button        
    }
}

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/content/circuit-panel.ui")]
pub struct CircuitPanelTemplate {
    #[template_child]
    pub header_bar: TemplateChild<adw::HeaderBar>,

    #[template_child]
    pub back_button: TemplateChild<gtk::Button>,

    #[template_child]
    pub view: TemplateChild<adw::TabView>,

    #[template_child]
    pub tab_bar: TemplateChild<adw::TabBar>,

    #[template_child]
    undo_button: TemplateChild<gtk::Button>,

    #[template_child]
    redo_button: TemplateChild<gtk::Button>,

    application: RefCell<Application>,
    pages: RefCell<HashMap<String, adw::TabPage>>
}

impl CircuitPanelTemplate {
    fn add_page<'a>(&self, content: &CircuitView, title: &'a str) -> adw::TabPage {
        let page = self.view.add_page(content, None);
        page.set_indicator_activatable(true);
        page.set_title(title);
        page
    }

    pub fn set_application(&self, app: Application) {
        self.application.replace(app);
    }

    pub fn new_tab<'a>(&self, title: &'a str, plot_provider: PlotProvider) {
        let content = CircuitView::new(self.application.borrow().clone(), plot_provider);
        let page = self.add_page(&content, title);
        self.view.set_selected_page(&page);
        self.pages.borrow_mut().insert(title.to_owned(), page);
    }

    pub fn remove_tab(&self, module_name: &String) {
        if let Some(page) = self.pages.borrow().get(module_name) {
            self.view.close_page(page);
        }
    }

    pub fn set_title<'a>(&self, title: &'a str) {
        (self.header_bar.title_widget().unwrap().downcast_ref().unwrap() as &adw::WindowTitle).set_subtitle(title);
    }

    pub fn close_tabs(&self) {
        for i in (0..self.view.n_pages()).rev() {
            self.view.close_page(&self.view.nth_page(i));
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for CircuitPanelTemplate {
    const NAME: &'static str = "CircuitPanel";
    type Type = CircuitPanel;
    type ParentType = gtk::Box;

    fn class_init(class: &mut Self::Class) {
        class.bind_template();
        class.bind_template_instance_callbacks();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for CircuitPanelTemplate {
    fn constructed(&self) {
        self.parent_constructed();
       // self.new_tab("Main");
       // self.new_tab("Second");
    }
}

impl WidgetImpl for CircuitPanelTemplate {}
impl BoxImpl for CircuitPanelTemplate {}
