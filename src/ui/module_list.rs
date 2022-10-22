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
    gdk,
    prelude::InitializingWidgetExt,
    subclass::{
        prelude::{BoxImpl, WidgetImpl},
        widget::{CompositeTemplate, WidgetClassSubclassExt},
    },
    Accessible, Box, Buildable, Button, CompositeTemplate, ConstraintTarget, Orientable,
    TemplateChild, Widget, ListBox, ListBoxRow, Label, traits::WidgetExt, GestureClick
};

use std::sync::Arc;

use crate::{
    application::Application,
    modules::Module,
    simulator::block::Block
};
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
    pub list_box: TemplateChild<ListBox>
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

        dialogs::new(&self.new_module_button, (400, 70), dialogs::new_module);

        crate::APPLICATION_DATA.with(|data| {
            let data = data.borrow();
            for (_, v) in data.modules().iter() {
                self.list_box.append(&new_list_item(v.clone()));
            }
        });
    }
}

impl WidgetImpl for ModuleListTemplate {}
impl BoxImpl for ModuleListTemplate {}

fn new_list_item(module: Arc<Module>) -> ListBoxRow {
    let label = Label::builder()
        .label(module.get_name().as_str())
        .build();
    
    let item = ListBoxRow::builder()
        .child(&label)
        .build();
        
    let click_gesture = GestureClick::builder()
        .button(gdk::ffi::GDK_BUTTON_PRIMARY as u32)
        .build();
        
    click_gesture.connect_pressed(move |_, _, _, _| {
        crate::APPLICATION_DATA.with(|data| {
            let mut data = data.borrow_mut();
            data.add_block(Block::new(module.clone(), (0, 0)));
        });
    });
    
    item.add_controller(&click_gesture);
    item.add_css_class("module-list-item");
    
    item
}