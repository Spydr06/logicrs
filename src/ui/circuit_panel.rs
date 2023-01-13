use crate::{application::Application, simulator::PlotProvider};
use super::circuit_view::CircuitView;
use gtk::{prelude::*, subclass::prelude::*, gio, glib};

use std::cell::RefCell;

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
    pub fn on_open_button_activate(&self, _btn: &gtk::Button) {
        self.imp().application.borrow().open();
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
    pub open_button: TemplateChild<gtk::Button>,

    #[template_child]
    pub view: TemplateChild<adw::TabView>,

    #[template_child]
    pub tab_bar: TemplateChild<adw::TabBar>,

    application: RefCell<Application>
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

    pub fn new_tab<'a>(&self, title: &'a str, plot_provider: PlotProvider) -> adw::TabPage {
        let content = CircuitView::new(plot_provider);
        let page = self.add_page(&content, title);
        self.view.set_selected_page(&page);

        page
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
