use gtk::{prelude::*, subclass::prelude::*, gio, glib};
use adw::subclass::prelude::AdwApplicationWindowImpl;
use crate::{application::Application, modules::Module, simulator::PlotProvider};
use super::{
    circuit_panel::{CircuitPanel, CircuitPanelTemplate},
    module_list::{ModuleList, ModuleListTemplate}, circuit_view::CircuitView,
};

glib::wrapper! {
    pub struct MainWindow(ObjectSubclass<MainWindowTemplate>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl MainWindow {
    pub fn new(app: &Application) -> Self {
        let window: Self = glib::Object::new::<Self>(&[
                ("application", app),
                ("title", &"LogicRs"),
            ]);
        window.imp().set_application(app.clone());
        let module_list = window.imp().module_list.imp();
        module_list.initialize();

        let panel = window.imp().circuit_panel.imp();
        panel.set_title(&app.imp().file_name());
        panel.new_tab("Main Circuit", PlotProvider::Main(app.imp().project().clone()));

        let project = app.imp().project();
        let project = project.lock().unwrap();
        project.modules().iter().for_each(|(_, module)| window.add_module_to_ui(app, module));

        window
    }

    pub fn add_module_to_ui(&self, app: &Application, module: &Module) {
        let panel = self.imp().circuit_panel.imp();
        let module_list = &self.imp().module_list;
        if !module.builtin() {
            panel.new_tab(module.name(), PlotProvider::Module(app.imp().project().clone(), module.name().clone()));
        }
        module_list.add_module_to_ui(app, module);
    }

    pub fn rerender_circuit(&self) {
        if let Some(a) = self.imp().circuit_panel.imp().view.selected_page() {
            if let Ok(view ) = a.child().downcast::<CircuitView>() {
                view.imp().rerender();
            }
        }
    }
}

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/content/main-window.ui")]
pub struct MainWindowTemplate {
    #[template_child]
    pub leaflet: TemplateChild<adw::Leaflet>,

    #[template_child]
    pub module_list: TemplateChild<ModuleList>,

    #[template_child]
    pub circuit_panel: TemplateChild<CircuitPanel>,
}

impl MainWindowTemplate {
    pub fn set_application(&self, app: Application) {
        self.circuit_panel.get().imp().set_application(app);
    }
}

#[glib::object_subclass]
impl ObjectSubclass for MainWindowTemplate {
    const NAME: &'static str = "MainWindow";
    type Type = MainWindow;
    type ParentType = adw::ApplicationWindow;

    fn class_init(my_class: &mut Self::Class) {
        Self::bind_template(my_class);
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for MainWindowTemplate {
    fn constructed(&self) {
        self.parent_constructed();

    //    obj.set_title(Some("LogicRs"));

        let module_list = self.module_list.get();
        let module_list_template = ModuleListTemplate::from_instance(&module_list);

        let circuit_panel = self.circuit_panel.get();
        let circuit_panel_template = CircuitPanelTemplate::from_instance(&circuit_panel);

        self.leaflet.property_expression("folded").bind(
            &module_list_template.header_bar.get(),
            "show-end-title-buttons",
            gtk::Widget::NONE,
        );

        self.leaflet.property_expression("folded").bind(
            &circuit_panel_template.header_bar.get(),
            "show-start-title-buttons",
            gtk::Widget::NONE,
        );

        self.leaflet.property_expression("folded").bind(
            &circuit_panel_template.back_button.get(),
            "visible",
            gtk::Widget::NONE,
        );
    }
}

impl WidgetImpl for MainWindowTemplate {}
impl WindowImpl for MainWindowTemplate {}
impl ApplicationWindowImpl for MainWindowTemplate {}
impl AdwApplicationWindowImpl for MainWindowTemplate {}
