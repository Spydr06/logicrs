use gtk::{prelude::*, subclass::prelude::*, glib, gdk, gio};

use crate::{application::{Application, action::Action}, simulator::*, renderer::vector::Vector2};

macro_rules! add_menu_item {
    ($model: expr, $name: expr, $action: expr, $value: expr) => {
        let __item__ = gtk::gio::MenuItem::new(Some($name), Some($action));
        __item__.set_attribute_value("target", Some($value));
        $model.append_item(&__item__);
    };
}

trait ModuleListItem {
    fn label(&self) -> Option<glib::GString>;
}

impl ModuleListItem for gtk::ListBoxRow {
    fn label(&self) -> Option<glib::GString> {
        self.first_child()
            .and_then(|w| w.downcast::<gtk::Box>().ok())
            .and_then(|w| w.last_child())
            .and_then(|w| w.downcast::<gtk::Label>().ok())
            .map(|w| w.label())
    }
}

trait ModuleListBox {
    fn n_visible(&self) -> u32;
}

impl ModuleListBox for gtk::ListBox {
    fn n_visible(&self) -> u32 {
        let mut index = 0;
        let mut counter = 0;
        while let Some(child) = self.row_at_index(index) {
            index += 1;
            counter += child.is_child_visible() as u32;
        }
        counter
    }
}

glib::wrapper! {
    pub struct ModuleList(ObjectSubclass<ModuleListTemplate>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl ModuleList {
    pub fn init_accels(&self, app: &Application) {
        app.set_accels_for_action("list.search", &["<primary>F"]);
    }

    pub fn add_module_to_ui(&self, app: &Application, module: &Module) {
        self.imp().add_module_to_ui(app, module);
    }

    pub fn remove_module_from_ui(&self, module_name: &String) {
        self.imp().remove_module_from_ui(module_name);
    }

    pub fn clear_list(&self) {
        self.imp().clear_list();
    }

    pub fn show_search(&self) {
        self.imp().search_bar.set_search_mode(true);
    }
}

#[gtk::template_callbacks]
impl ModuleList {
    #[template_callback]
    fn search_entry_started(&self, entry: &gtk::SearchEntry) {
        entry.grab_focus();
    }

    #[template_callback]
    fn search_entry_changed(&self, entry: &gtk::SearchEntry) {
        let search_text = entry.text().to_ascii_lowercase();
        self.imp().filter((!search_text.is_empty()).then_some(search_text));
    }

    #[template_callback]
    fn search_entry_stopped(&self, entry: &gtk::SearchEntry) {
        entry.set_text("");
    }
}

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/content/module-list.ui")]
pub struct ModuleListTemplate {
    #[template_child]
    pub header_bar: TemplateChild<adw::HeaderBar>,

    #[template_child]
    stack: TemplateChild<gtk::Stack>,

    #[template_child]
    basic_list_box: TemplateChild<gtk::ListBox>,

    #[template_child]
    input_output_list_box: TemplateChild<gtk::ListBox>,

    #[template_child]
    gate_list_box: TemplateChild<gtk::ListBox>,

    #[template_child]
    latch_list_box: TemplateChild<gtk::ListBox>,

    #[template_child]
    flip_flop_list_box: TemplateChild<gtk::ListBox>,

    #[template_child]
    custom_list_box: TemplateChild<gtk::ListBox>,

    #[template_child]
    search_bar: TemplateChild<gtk::SearchBar>,

    #[template_child]
    search_button: TemplateChild<gtk::ToggleButton>
}

impl ModuleListTemplate {
    fn list_for(&self, category: Category) -> &gtk::ListBox {
        match category {
            Category::Basic => &self.basic_list_box,
            Category::InputOutput => &self.input_output_list_box,
            Category::Gate => &self.gate_list_box,
            Category::Latch => &self.latch_list_box,
            Category::FlipFlop => &self.flip_flop_list_box,
            Category::Custom => &self.custom_list_box,
            Category::Hidden => panic!("no list for hidden modules")
        }
    }

    fn lists(&self) -> [&gtk::ListBox; 6] {
        [&self.basic_list_box, &self.input_output_list_box, &self.gate_list_box, &self.latch_list_box, &self.flip_flop_list_box, &self.custom_list_box]
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
        
        self.list_for(module.category())
            .append(&item);
            
        let left_click_gesture = gtk::GestureClick::builder()
            .button(gdk::ffi::GDK_BUTTON_PRIMARY as u32)
            .build();
        
        let name = module.name().to_owned();
        left_click_gesture.connect_pressed(glib::clone!(@weak application => move |_, _, _, _| {
            let project = application.imp().project().clone();
            let project = project.lock().unwrap();
            if let Some(module) = project.module(&name) && let Some(plot) = application.imp().current_plot() {
                let block = Block::new(&module, Vector2(0, 0));
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

    fn clear_list(&self) {
        self.lists().iter().for_each(|list| 
            while let Some(row) = list.row_at_index(0) {
                list.remove(&row);
            }
        );
    }

    fn custom_module_context(&self, item: &gtk::ListBoxRow, name: &String) {
        let model = gio::Menu::new();
        add_menu_item!(model, "_Edit Contents", "app.edit-module",   &name.to_variant());
        add_menu_item!(model, "E_xport",        "app.export-module", &name.to_variant());
        add_menu_item!(model, "_Delete",        "app.delete-module", &name.to_variant());

        let popover = gtk::PopoverMenu::from_model(Some(&model));
        popover.set_parent(item);
        popover.popup();
    }

    fn n_visible(&self) -> u32 {
        self.lists().iter().map(|list| list.n_visible()).sum()
    }

    fn filter(&self, search_text: Option<String>) {
        if let Some(search_text) = search_text {
            let search_text_copy = search_text.clone();
            self.lists().iter().for_each(move |list| {
                let search_text_copy = search_text_copy.clone();
                list.set_filter_func(move |item| Self::filter_func(item, &search_text_copy));
            });
        }
        else {
            self.lists().iter().for_each(|list| list.unset_filter_func());
        }

        self.stack.set_visible_child_name(if self.n_visible() == 0 { "empty" } else { "modules" });
    }

    fn filter_func(item: &gtk::ListBoxRow, search_text: &String) -> bool {
        let label = item.label().expect("could not get label from ModuleListItem");
        label.to_ascii_lowercase().contains(search_text)
    }
}

#[glib::object_subclass]
impl ObjectSubclass for ModuleListTemplate {
    const NAME: &'static str = "ModuleList";
    type Type = ModuleList;
    type ParentType = gtk::Box;

    fn class_init(class: &mut Self::Class) {
        class.bind_template();
        class.bind_template_instance_callbacks();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for ModuleListTemplate {
    fn constructed(&self) {
        self.parent_constructed();

        self.search_bar.connect_search_mode_enabled_notify(glib::clone!(@weak self as widget => move |search_bar|
            widget.search_button.set_active(search_bar.is_search_mode());
        ));

        self.search_button.connect_toggled(glib::clone!(@weak self as widget => move |button|
            widget.search_bar.set_search_mode(button.is_active());
        ));

        let order_alphabetically = |a: &gtk::ListBoxRow, b: &gtk::ListBoxRow| gtk::Ordering::from(
            a.label().expect("could not get label from ModuleListItem")
            .cmp(&b.label().expect("could not get label from ModuleListItem"))
        );
        self.lists().iter().for_each(|list| list.set_sort_func(order_alphabetically));
    }
}

impl WidgetImpl for ModuleListTemplate {}
impl BoxImpl for ModuleListTemplate {}
