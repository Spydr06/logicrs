use adw::prelude::*;
use gtk::{
    traits::DialogExt,
    subclass::prelude::ObjectSubclassIsExt,
    ButtonsType, Entry, MessageDialog, ResponseType, ComboBoxText, Orientation, Box, 
};

use std::future::Future;
use crate::{modules::Module, application::Application};

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
    app.imp().add_module(Module::new(name, num_inputs, num_outputs));

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

const OUTPUTS: [&'static str; 16] = [
    "1 Output", "2 Outputs", "3 Outputs", "4 Outputs",
    "5 Outputs", "6 Outputs", "7 Outputs", "8 Outputs",
    "9 Outputs", "10 Outputs", "11 Outputs", "12 Outputs",
    "13 Outputs", "14 Outputs", "15 Outputs", "16 Outputs",
];

const INPUTS: [&'static str; 16] = [
    "1 Input", "2 Inputs", "3 Inputs", "4 Inputs",
    "5 Inputs", "6 Inputs", "7 Inputs", "8 Inputs",
    "9 Inputs", "10 Inputs", "11 Inputs", "12 Inputs",
    "13 Inputs", "14 Inputs", "15 Inputs", "16 Inputs",
];

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
        .build();
        content.append(&name_input);

    let input_chooser = ComboBoxText::builder()
        .sensitive(true)
        .margin_start(12)
        .tooltip_text("Number of input pins")
        .build();
    INPUTS.iter().for_each(|&elem| { input_chooser.append(Some(elem), elem) });
    input_chooser.set_active_id(Some(INPUTS[1]));
    content.append(&input_chooser);

    let output_chooser = ComboBoxText::builder()        
        .sensitive(true)
        .margin_start(12)
        .tooltip_text("Number of ouput pins")
        .build();
    OUTPUTS.iter().for_each(|&elem| { output_chooser.append(Some(elem), elem) });
    output_chooser.set_active_id(Some(OUTPUTS[0]));
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
        let num_inputs = INPUTS.iter().position(|&elem| elem == input_chooser.active_id().unwrap()).unwrap_or_default() + 1;
        let num_outputs = OUTPUTS.iter().position(|&elem| elem == output_chooser.active_id().unwrap()).unwrap_or_default() + 1;

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

pub fn run <F, T>(application: Application, window: gtk::Window, data: T, dialog: fn(Application, gtk::Window, T) -> F) 
where
    F: Future<Output = ()> + 'static,
{
    gtk::glib::MainContext::default().spawn_local(dialog(application, window.clone(), data));
}
