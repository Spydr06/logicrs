use adw::{self, HeaderBar};

use glib::{
    clone, object_subclass,
    subclass::{
        object::{ObjectImpl, ObjectImplExt},
        types::ObjectSubclass,
        InitializingObject,
    },
    wrapper, IsA,
};

use gtk::{
    prelude::InitializingWidgetExt,
    subclass::{
        prelude::{BoxImpl, WidgetImpl},
        widget::{CompositeTemplate, WidgetClassSubclassExt},
    },
    traits::{ButtonExt, GtkApplicationExt, GtkWindowExt},
    Accessible, Box, Buildable, Button, CompositeTemplate, ConstraintTarget, Inhibit, Orientable,
    TemplateChild, Widget,
};

use adw::prelude::DialogExtManual;
use std::{cell::RefCell, rc::Rc};

use crate::application::{self, Application};

wrapper! {
    pub struct ModuleList(ObjectSubclass<ModuleListTemplate>)
        @extends Widget, Box,
        @implements Accessible, Buildable, ConstraintTarget, Orientable;
}

impl Default for ModuleList {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleList {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create an instance of ModuleList")
    }
}

#[derive(CompositeTemplate, Default)]
#[template(resource = "/app/module-list.ui")]
pub struct ModuleListTemplate {
    #[template_child]
    pub header_bar: TemplateChild<HeaderBar>,

    #[template_child]
    pub new_module_button: TemplateChild<Button>,
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

        let dialog_window = Rc::new(
            gtk::ApplicationWindow::builder()
                .title("New Module")
                .default_width(350)
                .default_height(70)
                .visible(false)
                .resizable(false)
                .build(),
        );

        self.new_module_button
            .get()
            .connect_clicked(clone!(@strong dialog_window =>
                move |_| {
                    gtk::glib::MainContext::default().spawn_local(new_module_dialog(Rc::clone(&dialog_window)));
                }
            ));

        dialog_window.connect_close_request(move |dialog_window| {
            if let Some(application) = dialog_window.application() {
                application.remove_window(dialog_window);
            }
            Inhibit(false)
        });
    }
}

impl WidgetImpl for ModuleListTemplate {}
impl BoxImpl for ModuleListTemplate {}

async fn new_module_dialog<W: IsA<gtk::Window>>(window: Rc<W>) {
    let question_dialog = gtk::MessageDialog::builder()
        .transient_for(&*window)
        .modal(true)
        .buttons(gtk::ButtonsType::OkCancel)
        .text("Create a New Module")
        .resizable(false)
        .build();

    let answer = question_dialog.run_future().await;
    question_dialog.close();

    println!("Answer: {}", answer);
}
