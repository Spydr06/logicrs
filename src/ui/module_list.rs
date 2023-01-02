use gtk::{prelude::*, subclass::prelude::*, glib, gdk};

use std::cell::RefCell;

use crate::{application::Application, modules::Module, simulator::Block};
use super::dialogs;

glib::wrapper! {
    pub struct ModuleList(ObjectSubclass<ModuleListTemplate>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl ModuleList {
    pub fn new(app: &Application) -> Self {
        glib::Object::new::<Self>(&[("application", app)])
    }
}

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/content/module-list.ui")]
pub struct ModuleListTemplate {
    #[template_child]
    pub header_bar: TemplateChild<adw::HeaderBar>,

    #[template_child]
    pub new_module_button: TemplateChild<gtk::Button>,

    #[template_child]
    pub builtin_list_box: TemplateChild<gtk::ListBox>,

    #[template_child]
    pub custom_list_box: TemplateChild<gtk::ListBox>,
    application: RefCell<Application>
}

impl ModuleListTemplate {
    pub fn get_from(parent: &ModuleList) -> &Self {
        Self::from_instance(parent)
    }

    pub fn set_application(&self, app: Application) {
        self.application.replace(app);
    }

    pub fn application_data(&self) -> Application {
        self.application.borrow().clone()
    }

    fn new_list_item(&self, module: &Module) {
        let label = gtk::Label::builder()
            .label(module.name().as_str())
            .build();
        
        let item = gtk::ListBoxRow::builder()
            .child(&label)
            .build();
            
        let click_gesture = gtk::GestureClick::builder()
            .button(gdk::ffi::GDK_BUTTON_PRIMARY as u32)
            .build();
        
        let name = module.name().to_owned();
        let data = self.application.borrow().imp().data();
        click_gesture.connect_pressed(move |_, _, _, _| {
                let mut data = data.lock().unwrap();
                let module = data.get_module(&name).unwrap();
                let block = Block::new(&module, (0, 0), data.new_id());
                data.current_plot_mut().add_block(block);
        });
        
        item.add_controller(&click_gesture);
        item.add_css_class("module-list-item");
        
        if module.builtin() {
            self.builtin_list_box.append(&item);
        }
        else {
            self.custom_list_box.append(&item);
        }
    }

    pub fn initialize(&self) {
        let data = self.application.borrow().imp().data();
        dialogs::new(data.clone(), &self.new_module_button, (400, 70), dialogs::new_module);
        
        let data = data.lock().unwrap();
        let mut values: Vec<_> = data.modules().values().into_iter().collect();
        values.sort();
        values.iter().for_each(|m| self.new_list_item(m));
    
        let order_alphabetically = |a: &gtk::ListBoxRow, b: &gtk::ListBoxRow| gtk::Ordering::from(
            (a.first_child().unwrap().downcast_ref().unwrap() as &gtk::Label).label()
            .cmp(&(b.first_child().unwrap().downcast_ref().unwrap() as &gtk::Label).label())
        );
        self.builtin_list_box.set_sort_func(order_alphabetically);
        self.custom_list_box.set_sort_func(order_alphabetically);
    }
}

#[glib::object_subclass]
impl ObjectSubclass for ModuleListTemplate {
    const NAME: &'static str = "ModuleList";
    type Type = ModuleList;
    type ParentType = gtk::Box;

    fn class_init(my_class: &mut Self::Class) {
        Self::bind_template(my_class);
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for ModuleListTemplate {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

impl WidgetImpl for ModuleListTemplate {}
impl BoxImpl for ModuleListTemplate {}
