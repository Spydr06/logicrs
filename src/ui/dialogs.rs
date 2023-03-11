use adw::prelude::*;
use gtk::{
    traits::DialogExt,
    subclass::prelude::ObjectSubclassIsExt,
    ButtonsType, Entry, MessageDialog, ResponseType, Orientation, Box, 
};

use std::future::Future;
use crate::{simulator::Module, application::{Application, action::Action}};

fn create_new_module(app: Application, name: String, num_inputs: u8, num_outputs: u8) -> Result<(), String> {
    if name.is_empty() {
        return Err("Invalid name".to_string());
    }

    if app.imp().project().lock().unwrap().module(&name).is_some() {
        let err = format!("Module with name \"{}\" already exists", name);
        warn!("{err}");
        return Err(err);
    }

    info!("Create new Module \"{}\"\nwith: {} inputs\n      {} outputs", name, num_inputs, num_outputs);
    app.new_action(Action::CreateModule(app.imp().project().clone(), Module::new(name, num_inputs, num_outputs)));

    Ok(())
}

pub async fn invalid_module(window: gtk::Window, msg: String) {
    let dialog = MessageDialog::builder()
        .transient_for(&window)
        .modal(true)
        .buttons(ButtonsType::Ok)
        .text(format!("Error creating module: {}", msg).as_str())
        .resizable(false)
        .build();

    dialog.run_future().await;
    dialog.close();
}

pub async fn new_module(app: Application, window: gtk::Window, _data: ()) {
    let content = Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .margin_start(12)
        .margin_end(12)
        .build();

    let name_input = Entry::builder()
        .text("New Module")
        .hexpand(true)
        .max_length(Module::MAX_MODULE_NAME_LEN)
        .overwrite_mode(true)
        .build();
        content.append(&name_input);

    let input_adjustment = gtk::Adjustment::new(2.0, 1.0, 129.0, 1.0, 1.0, 1.0);
    let input_chooser = gtk::SpinButton::builder()
        .climb_rate(1.0)
        .adjustment(&input_adjustment)
        .margin_start(12)
        .text("Inputs")
        .numeric(true)
        .tooltip_text("Select the number of input pins.")
        .build();
    content.append(&input_chooser);

    let output_adjustment = gtk::Adjustment::new(1.0, 1.0, 129.0, 1.0, 1.0, 1.0);
    let output_chooser = gtk::SpinButton::builder()        
        .climb_rate(1.0)
        .adjustment(&output_adjustment)
        .margin_start(12)
        .text("Outputs")
        .numeric(true)
        .tooltip_text("Select the number of output pins.")
        .build();
    content.append(&output_chooser);

    let dialog = MessageDialog::builder()
        .transient_for(&window)
        .modal(true)
        .buttons(ButtonsType::OkCancel)
        .text("Create a New Module")
        .resizable(false)
        .build();

    dialog.content_area().append(&name_input);
    dialog.content_area().append(&content);

    let answer = dialog.run_future().await;
    dialog.close();

    if answer == ResponseType::Ok {
       // let num_inputs = INPUTS.iter().position(|&elem| elem == input_chooser.active_id().unwrap()).unwrap_or_default() + 1;
        let num_inputs = input_chooser.value_as_int();
        let num_outputs = output_chooser.value_as_int();

        // generate new module
        if let Err(err) = create_new_module(app, name_input.buffer().text().trim().to_string(), num_inputs as u8, num_outputs as u8) {
            gtk::glib::MainContext::default().spawn_local(invalid_module(window, err));
        }
    }
}

pub async fn basic_error(_app: Application, window: gtk::Window, message: String) {
    let dialog = MessageDialog::builder()
        .transient_for(&window)
        .modal(true)
        .buttons(ButtonsType::Ok)
        .resizable(false)
        .text(&message)
        .title("Error")
        .build();
    
    dialog.run_future().await;
    dialog.close();
}

pub async fn confirm_delete_module(app: Application, window: gtk::Window, module_name: String) {
    let dialog = MessageDialog::builder()
        .transient_for(&window)
        .modal(true)
        .buttons(ButtonsType::YesNo)
        .resizable(false)
        .text(&format!("Do you really want to delete the module \"{module_name}\"?\nThis action is not reversable!"))
        .title(&format!("Delete Module \"{module_name}\"?"))
        .build();
    
    let answer = dialog.run_future().await;
    dialog.close();

    if ResponseType::Yes == answer {
        app.imp().delete_module(&module_name);
    }
}

pub fn run<F, T>(application: Application, window: gtk::Window, data: T, dialog: fn(Application, gtk::Window, T) -> F) 
where
    F: Future<Output = ()> + 'static,
{
    gtk::glib::MainContext::default().spawn_local(dialog(application, window.clone(), data));
}
