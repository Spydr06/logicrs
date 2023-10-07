use adw::prelude::*;
use gtk::{
    subclass::prelude::ObjectSubclassIsExt, traits::DialogExt, Align, Box, ButtonsType,
    ColorButton, Entry, Label, MessageDialog, Orientation, ResponseType,
};

use crate::{
    application::{action::Action, selection::SelectionField, Application},
    renderer::{IntoColor, IntoRGBA, COLOR_THEME},
    simulator::Module,
};
use std::future::Future;

fn create_new_module(
    app: Application,
    name: String,
    num_inputs: u8,
    num_outputs: u8,
) -> Result<(), String> {
    if name.is_empty() {
        return Err("Invalid name".to_string());
    }

    if app.imp().project().lock().unwrap().module(&name).is_some() {
        let err = format!("Module with name \"{}\" already exists", name);
        warn!("{err}");
        return Err(err);
    }

    info!(
        "Create new Module \"{}\"\nwith: {} inputs\n      {} outputs",
        name, num_inputs, num_outputs
    );
    app.new_action(Action::CreateModule(
        app.imp().project().clone(),
        Module::new(name, num_inputs, num_outputs),
    ));

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
        if let Err(err) = create_new_module(
            app,
            name_input.buffer().text().trim().to_string(),
            num_inputs as u8,
            num_outputs as u8,
        ) {
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

    if answer == ResponseType::Yes {
        app.imp().delete_module(&module_name);
    }
}

pub async fn select_border_color(app: Application, window: gtk::Window, _data: ()) {
    let dialog = MessageDialog::builder()
        .transient_for(&window)
        .modal(true)
        .resizable(false)
        .title("Select Border Color")
        .buttons(ButtonsType::OkCancel)
        .build();

    let label = Label::builder()
        .label("Border Color:")
        .halign(Align::Start)
        .hexpand(false)
        .build();
    let color_button = ColorButton::with_rgba(&unsafe { &COLOR_THEME.border_color }.into_rgba());

    let content = dialog.content_area();
    content.set_orientation(Orientation::Horizontal);
    content.set_hexpand(true);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_halign(Align::Start);
    content.append(&label);
    content.append(&color_button);

    let answer = dialog.run_future().await;
    dialog.close();

    if answer == ResponseType::Ok && let Some(plot_provider) = app.imp().current_plot() && let Some(block_ids) = plot_provider.with_mut(|plot| plot.selection().blocks()) {
        let color = color_button.rgba().into_color();
        println!("here");
        app.new_action(Action::ChangeBorderColor(plot_provider, color, block_ids, vec![]));
    }
}

pub fn run<F, T>(
    application: Application,
    window: gtk::Window,
    data: T,
    dialog: fn(Application, gtk::Window, T) -> F,
) where
    F: Future<Output = ()> + 'static,
{
    gtk::glib::MainContext::default().spawn_local(dialog(application, window, data));
}
