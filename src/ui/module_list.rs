use gtk::{prelude::*, subclass::prelude::*, glib, gdk, gio};

use crate::{application::{Application, action::Action}, simulator::*};

glib::wrapper! {
    pub struct ModuleList(ObjectSubclass<ModuleListTemplate>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl ModuleList {
    pub fn new(app: &Application) -> Self {
        glib::Object::new::<Self>(&[("application", app)])
    }

    pub fn add_module_to_ui(&self, app: &Application, module: &Module) {
        self.imp().add_module_to_ui(app, module);
    }

    pub fn remove_module(&self, module: &Module) {
        self.imp().remove_module(module);
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

    fn add_module_to_ui(&self, application: &Application, module: &Module) {
        let item = gtk::ListBoxRow::builder()
            .child(&gtk::Label::new(Some(&module.name())))
            .css_classes(vec![String::from("module-list-item")])
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
        right_click_gesture.connect_pressed(glib::clone!(@weak self as widget => move |_, _, x, y| {
            if !is_builtin {
                widget.custom_module_context(&item, &name, x as i32, y as i32);
            }
        }));
    }

    fn remove_module(&self, module: &Module) {
        let list = if module.builtin() { &self.builtin_list_box } else { &self.custom_list_box };
        let mut i = 0;
        while let Some(row) = list.row_at_index(i) {
            if let Some(label) = row.child() {
                if label.downcast::<gtk::Label>().unwrap_or_default().label().to_string().eq(module.name()) {
                    list.remove(&row);
                    break;
                }
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

    fn custom_module_context(&self, item: &gtk::ListBoxRow, _name: &String, x: i32, y: i32) {
        let model = gio::MenuModel::NONE; // TODO

        let popover = gtk::PopoverMenu::from_model(model);
        popover.set_parent(item);
        popover.set_pointing_to(Some(&gdk::Rectangle::new(x, y, 1, 1)));
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
            (a.first_child().unwrap().downcast_ref().unwrap() as &gtk::Label).label()
            .cmp(&(b.first_child().unwrap().downcast_ref().unwrap() as &gtk::Label).label())
        );
        self.builtin_list_box.set_sort_func(order_alphabetically);
        self.custom_list_box.set_sort_func(order_alphabetically);
    }
}

impl WidgetImpl for ModuleListTemplate {}
impl BoxImpl for ModuleListTemplate {}
