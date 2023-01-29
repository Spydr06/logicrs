use gtk::{prelude::*, subclass::prelude::*, glib, gdk, gio};

use crate::{application::{Application, action::Action}, simulator::*};

glib::wrapper! {
    pub struct ModuleList(ObjectSubclass<ModuleListTemplate>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl ModuleList {
    pub fn add_module_to_ui(&self, app: &Application, module: &Module) {
        self.imp().add_module_to_ui(app, module);
    }

    pub fn remove_module_from_ui(&self, module_name: &String) {
        self.imp().remove_module_from_ui(module_name);
    }
}

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/content/module-list.ui")]
pub struct ModuleListTemplate {
    #[template_child]
    pub header_bar: TemplateChild<adw::HeaderBar>,

    #[template_child]
    builtin_list_box: TemplateChild<gtk::ListBox>,

    #[template_child]
    custom_list_box: TemplateChild<gtk::ListBox>,
}

impl ModuleListTemplate {
    pub fn get_from(parent: &ModuleList) -> &Self {
        Self::from_instance(parent)
    }

    fn module_item_content(&self, module: &Module) -> gtk::Box {        
        let b = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .build();

        b.append(&gtk::Image::builder().icon_name("module-symbolic").margin_end(12).build());
        b.append(&gtk::Label::builder()
            .label(&module.name())
            .xalign(0.0)
            .build()
        );

        b
    }

    fn add_module_to_ui(&self, application: &Application, module: &Module) {
        let item = gtk::ListBoxRow::builder()
            .child(&self.module_item_content(module))
            .css_classes(vec![String::from("module_list_item")])
            .build();
        
        module.builtin()
            .then_some(&self.builtin_list_box)
            .unwrap_or(&self.custom_list_box)
            .append(&item);
            
        let left_click_gesture = gtk::GestureClick::builder()
            .button(gdk::ffi::GDK_BUTTON_PRIMARY as u32)
            .build();
        
        let name = module.name().to_owned();
        left_click_gesture.connect_pressed(glib::clone!(@weak application => move |_, _, _, _| {
            let id = application.imp().with_current_plot_mut(|plot| plot.next_id()).unwrap();
            let project = application.imp().project().clone();
            let project = project.lock().unwrap();
            if let Some(module) = project.module(&name) && let Some(plot) = application.imp().current_plot() {
                let block = Block::new(&module, (0, 0), id);
                drop(project);
                application.new_action(Action::NewBlock(plot, block));
            }
        }));
        item.add_controller(&left_click_gesture);

        let right_click_gesture = gtk::GestureClick::builder()
            .button(gdk::ffi::GDK_BUTTON_SECONDARY as u32)
            .build();
        item.add_controller(&right_click_gesture);

        let name = module.name().to_owned();
        let is_builtin = module.builtin();
        right_click_gesture.connect_pressed(glib::clone!(@weak self as widget => move |_, _, _, _| {
            if !is_builtin {
                widget.custom_module_context(&item, &name);
            }
        }));
    }

    fn remove_module_from_ui(&self, module_name: &String) {
        let mut i = 0;
        while let Some(row) = self.custom_list_box.row_at_index(i) {
            if row.child().and_downcast::<gtk::Box>().unwrap().last_child().and_downcast::<gtk::Label>().unwrap().label().eq(module_name) {
                self.custom_list_box.remove(&row);
            }
            i += 1;
        }
    }

    pub fn clear_list(&self) {
        [&self.builtin_list_box, &self.custom_list_box].iter().for_each(|list| 
            while let Some(row) = list.row_at_index(0) {
                list.remove(&row);
            }
        );
    }

    fn custom_module_context(&self, item: &gtk::ListBoxRow, name: &String) {
        let delete_item = gio::MenuItem::new(Some("_Delete"), Some("app.delete-module"));
        delete_item.set_attribute_value("target", Some(&name.to_variant()));

        let export_item = gio::MenuItem::new(Some("_Export"), Some("app.export-module"));
        export_item.set_attribute_value("target", Some(&name.to_variant()));

        let model = gio::Menu::new();
        model.append_item(&delete_item);
        model.append_item(&export_item);

        let popover = gtk::PopoverMenu::from_model(Some(&model));
        popover.set_parent(item);
        popover.popup();
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

        let order_alphabetically = |a: &gtk::ListBoxRow, b: &gtk::ListBoxRow| gtk::Ordering::from(
            ((a.first_child().unwrap().downcast_ref().unwrap() as &gtk::Box).last_child().unwrap().downcast_ref().unwrap() as &gtk::Label).label()
            .cmp(&((b.first_child().unwrap().downcast_ref().unwrap() as &gtk::Box).last_child().unwrap().downcast_ref().unwrap() as &gtk::Label).label())
        );
        self.builtin_list_box.set_sort_func(order_alphabetically);
        self.custom_list_box.set_sort_func(order_alphabetically);
    }
}

impl WidgetImpl for ModuleListTemplate {}
impl BoxImpl for ModuleListTemplate {}
