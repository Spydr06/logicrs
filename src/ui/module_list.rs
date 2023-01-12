use gtk::{prelude::*, subclass::prelude::*, glib, gdk};

use crate::{application::Application, modules::Module, simulator::Block};

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
    pub builtin_list_box: TemplateChild<gtk::ListBox>,

    #[template_child]
    pub custom_list_box: TemplateChild<gtk::ListBox>
}

impl ModuleListTemplate {
    pub fn get_from(parent: &ModuleList) -> &Self {
        Self::from_instance(parent)
    }

    fn add_module_to_ui(&self, application: &Application, module: &Module) {
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
        let project = application.imp().project().clone();
        click_gesture.connect_pressed(glib::clone!(@weak application => move |_, _, _, _| {
                let mut project = project.lock().unwrap();
                let id = project.new_id();
                if let Some(module) = project.module(&name) {
                    println!("here");
                    let block = Block::new(&module, (0, 0), id);
                    drop(module);
                    drop(project);

                    application.imp().with_current_plot_mut(move |plot| plot.add_block(block.clone()));
                }
        }));
        
        item.add_controller(&click_gesture);
        item.add_css_class("module-list-item");
        
        if module.builtin() { &self.builtin_list_box } else { &self.custom_list_box }.append(&item);
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

    pub fn initialize(&self) {
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
