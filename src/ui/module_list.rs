use adw::{self, HeaderBar};

use glib::{
    object_subclass,
    subclass::{
        object::{ObjectImpl, ObjectImplExt},
        types::ObjectSubclass,
        types::ObjectSubclassExt,
        InitializingObject,
    },
    wrapper, Cast,
};

use gtk::{
    gdk,
    prelude::InitializingWidgetExt,
    subclass::{
        prelude::{BoxImpl, WidgetImpl},
        widget::{CompositeTemplate, WidgetClassSubclassExt},
    },
    Accessible, Box, Buildable, Button, CompositeTemplate, ConstraintTarget, Orientable,
    TemplateChild, Widget, ListBox, ListBoxRow, Label, traits::WidgetExt, GestureClick, Ordering
};

use std::cell::RefCell;

use crate::{application::{Application, data::ApplicationDataRef}, modules::Module, simulator::Block};
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
#[template(resource = "/content/module-list.ui")]
pub struct ModuleListTemplate {
    #[template_child]
    pub header_bar: TemplateChild<HeaderBar>,

    #[template_child]
    pub new_module_button: TemplateChild<Button>,

    #[template_child]
    pub builtin_list_box: TemplateChild<ListBox>,

    #[template_child]
    pub custom_list_box: TemplateChild<ListBox>,

    data: RefCell<ApplicationDataRef>
}

impl ModuleListTemplate {
    pub fn get_from(parent: &ModuleList) -> &Self {
        Self::from_instance(parent)
    }

    pub fn set_data(&self, data: ApplicationDataRef) {
        self.data.replace(data);
    }

    pub fn application_data(&self) -> ApplicationDataRef {
        self.data.borrow().clone()
    }

    fn new_list_item(&self, module: &Module) {
        let label = Label::builder()
            .label(module.name().as_str())
            .build();
        
        let item = ListBoxRow::builder()
            .child(&label)
            .build();
            
        let click_gesture = GestureClick::builder()
            .button(gdk::ffi::GDK_BUTTON_PRIMARY as u32)
            .build();
        
        let name = module.name().to_owned();
        let data = self.data.borrow().clone();
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
        let data = self.data.borrow();
        dialogs::new(data.clone(), &self.new_module_button, (400, 70), dialogs::new_module);
        
        let data = data.lock().unwrap();
        let mut values: Vec<_> = data.modules().values().into_iter().collect();
        values.sort();
        values.iter().for_each(|m| self.new_list_item(m));
    
        let order_alphabetically = |a: &ListBoxRow, b: &ListBoxRow| Ordering::from(
            (a.first_child().unwrap().downcast_ref().unwrap() as &Label).label()
            .cmp(&(b.first_child().unwrap().downcast_ref().unwrap() as &Label).label())
        );
        self.builtin_list_box.set_sort_func(order_alphabetically);
        self.custom_list_box.set_sort_func(order_alphabetically);
    }
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
    }
}

impl WidgetImpl for ModuleListTemplate {}
impl BoxImpl for ModuleListTemplate {}
