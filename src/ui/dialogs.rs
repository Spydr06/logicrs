use adw::prelude::{
    BoxExt, ButtonExt, DialogExtManual, EntryBufferExtManual, EntryExt, GtkWindowExt,
};
use glib::{clone, IsA};
use gtk::{
    traits::{DialogExt, GtkApplicationExt},
    Button, ButtonsType, Entry, Inhibit, MessageDialog, ResponseType,
};
use std::{future::Future, rc::Rc};

fn create_new_module(name: String) -> Result<(), &'static str> {
    if name.trim().is_empty() {
        return Err("Invalid name");
    }

    println!("Create new Module \"{}\"", name.trim());
    Ok(())
}

pub async fn invalid_module<W: IsA<gtk::Window>>(window: Rc<W>, msg: &'static str) {
    let dialog = MessageDialog::builder()
        .transient_for(&*window)
        .modal(true)
        .buttons(ButtonsType::Ok)
        .text(format!("Error creating module: {}", msg).as_str())
        .title("Module Error")
        .resizable(false)
        .build();

    dialog.run_future().await;
    dialog.close();
}

pub async fn new_module<W: IsA<gtk::Window>>(window: Rc<W>) {
    let name_input = Entry::builder()
        .text("New Module")
        .margin_start(12)
        .margin_end(12)
        .hexpand(true)
        .build();

    let dialog = MessageDialog::builder()
        .transient_for(&*window)
        .modal(true)
        .buttons(ButtonsType::OkCancel)
        .text("Create a New Module")
        .title("New Module")
        .resizable(false)
        .build();

    dialog.content_area().append(&name_input);

    let answer = dialog.run_future().await;
    dialog.close();

    match answer {
        ResponseType::Ok => {
            // generate new module
            if let Err(err) = create_new_module(name_input.buffer().text()) {
                gtk::glib::MainContext::default().spawn_local(invalid_module(window, err));
            }
            return;
        }
        ResponseType::Cancel => {
            // no new module will be generated
            return;
        }
        _ => {
            eprintln!("Unexpected ResponseType: {}", answer);
            return;
        }
    }
}

pub fn new<F>(trigger: &Button, window_size: (i32, i32), on_trigger: fn(Rc<gtk::ApplicationWindow>) -> F) 
where
    F: Future<Output = ()> + 'static,
{
    let dialog_window = Rc::new(
        gtk::ApplicationWindow::builder()
            .default_width(window_size.0)
            .default_height(window_size.1)
            .visible(false)
            .resizable(false)
            .build(),
    );

    trigger.connect_clicked(clone!(@strong dialog_window =>
        move |_| {
            gtk::glib::MainContext::default().spawn_local(on_trigger(Rc::clone(&dialog_window)));
        }
    ));

    dialog_window.connect_close_request(move |dialog_window| {
        if let Some(application) = dialog_window.application() {
            application.remove_window(dialog_window);
        }
        Inhibit(false)
    });
}
