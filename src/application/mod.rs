pub mod template;
pub mod actions;
pub mod action;
pub mod clipboard;
pub mod editor;

use action::*;
use std::cell::RefCell;
use adw::traits::MessageDialogExt;
use gtk::{prelude::*, subclass::prelude::*, gio, glib};
use crate::{config, ui::dialogs, selection::SelectionField, application::clipboard::Clipboard};

glib::wrapper! {
    pub struct Application(ObjectSubclass<template::ApplicationTemplate>)
    @extends gio::Application, gtk::Application, 
    @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl Application {
    pub fn new() -> Self {
        gio::resources_register_include!("logicrs.gresource").expect("Failed to register resources.");
        glib::Object::new::<Self>(&[
            ("application-id", &"com.spydr06.logicrs"),
            ("flags", &gio::ApplicationFlags::HANDLES_OPEN),
        ])
    }

    pub fn new_action(&self, action: Action) {
        self.imp().action_stack().borrow_mut().add(self, action);
    }

    pub fn undo_action(&self) {
        self.imp().action_stack().borrow_mut().undo(self);
    }

    pub fn redo_action(&self) {
        self.imp().action_stack().borrow_mut().redo(self);
    }

    pub fn apply_clipboard(&self, clipboard: Clipboard) {
        match clipboard {
            Clipboard::Blocks(_) => {
                match clipboard.paste_to(self.imp().current_plot().unwrap())
                {
                    Ok(action) => self.new_action(action),
                    Err(err) => dialogs::run(self.to_owned(), self.active_window().unwrap(), err, dialogs::basic_error)
                }
            }
            Clipboard::Module(_) => todo!(),
            Clipboard::Empty => {},
        }
    }

    pub fn paste_clipboard(&self) {
        let display = RootExt::display(&self.active_window().unwrap());
        display.clipboard().read_text_async(None as Option<&gio::Cancellable>, glib::clone!(@weak self as app => move |pasted| {
            match pasted
                .map_err(|err| err.to_string())
                .and_then(|text| text.ok_or(String::new()))
                .and_then(|text| Clipboard::deserialize(text.as_str()))
            {
                Ok(clipboard) => app.apply_clipboard(clipboard),
                Err(err) => warn!("Error pasting from clipboard: {err}")
            }
        }));
    }

    pub fn cut_clipboard(&self, clipboard: Clipboard) {
        match clipboard {
            Clipboard::Blocks(blocks) => self.new_action(Action::DeleteSelection(self.imp().current_plot().unwrap(), blocks, vec![])),
            Clipboard::Module(_) => todo!(),
            Clipboard::Empty => {}
        }
    }

    pub fn copy_clipboard(&self, cut: bool) {
        let display = RootExt::display(&self.active_window().unwrap());
        let clipboard = self.imp().generate_clipboard();

        match clipboard.serialize() {
            Ok(serialized) => {
                display.clipboard().set_text(&serialized);
                if cut {
                    self.cut_clipboard(clipboard);
                }
            }
            Err(err) => warn!("Error serializing clipboard: {err}")
        }
    }

    pub(self) fn setup_gactions(&self) {
        let quit_action = gio::SimpleAction::new("quit", None);
        quit_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            app.quit();
        }));
        self.add_action(&quit_action);

        let about_action = gio::SimpleAction::new("about", None);
        about_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            app.show_about();
        }));
        self.add_action(&about_action);

        let save_action = gio::SimpleAction::new("save", None);
        save_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            if let Err(err) = app.imp().save() {
                let message =  format!("Error saving to '{}': {}", app.imp().file_name(), err);
                error!("{}", message);
                if let Some(window) = app.active_window() {
                    dialogs::run(app, window, message, dialogs::basic_error);
                }
            }
        }));
        self.add_action(&save_action);

        let save_as_action = gio::SimpleAction::new("save-as", None);
        save_as_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            app.save_as();
        }));
        self.add_action(&save_as_action);

        let new_action = gio::SimpleAction::new("new", None);
        new_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            app.open_new();
        }));
        self.add_action(&new_action);

        let open_action = gio::SimpleAction::new("open", None);
        open_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            app.open();
        }));
        self.add_action(&open_action);

        let delete_block_action = gio::SimpleAction::new("delete-block", None);
        delete_block_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            if let Some(plot_provider) = app.imp().current_plot() {
                let blocks = plot_provider.with_mut(|plot| 
                    plot.selected().iter().map(|id| plot.get_block(*id).unwrap().to_owned()).collect()
                ).unwrap_or_default();
                app.new_action(Action::DeleteSelection(plot_provider, blocks, vec![]));
            }
        }));
        self.add_action(&delete_block_action);

        let create_new_module_action = gio::SimpleAction::new("create-new-module", None);
        create_new_module_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            if let Some(window) = app.active_window() {
                dialogs::run(app, window, (), dialogs::new_module); 
            }
        }));
        self.add_action(&create_new_module_action);

        let undo_action = gio::SimpleAction::new("undo", None);
        undo_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            app.undo_action();
        }));
        self.add_action(&undo_action);

        let redo_action = gio::SimpleAction::new("redo", None);
        redo_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            app.redo_action();
        }));
        self.add_action(&redo_action);

        let copy_action = gio::SimpleAction::new("copy", None);
        copy_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            app.copy_clipboard(false);
        }));
        self.add_action(&copy_action);

        let cut_action = gio::SimpleAction::new("cut", None);
        cut_action.connect_activate(glib::clone!(@weak self as app => move |_, _|  {
            app.copy_clipboard(true);
        }));
        self.add_action(&cut_action);

        let paste_action = gio::SimpleAction::new("paste", None);
        paste_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            app.paste_clipboard();
        }));
        self.add_action(&paste_action);

        let select_all_action = gio::SimpleAction::new("select-all", None);
        select_all_action.connect_activate(glib::clone!(@weak self as app => move |_, _| {
            app.imp().with_current_plot_mut(|plot| plot.select_all());
            app.imp().rerender_editor();
        }));
        self.add_action(&select_all_action);

        let delete_module_action = gio::SimpleAction::new("delete-module", Some(&String::static_variant_type()));
        delete_module_action.connect_activate(glib::clone!(@weak self as app => move |_, data| {
            let module_name = data
                .expect("Could not get module name target.")
                .get::<String>().unwrap();

            if let Some(window) = app.active_window() {
                dialogs::run(app, window, module_name, dialogs::confirm_delete_module);
            }
        }));
        self.add_action(&delete_module_action);

        /*let export_module_action = gio::SimpleAction::new("export-module", Some(&String::static_variant_type()));
        export_module_action.connect_activate(glib::clone!(@weak self as app => move |_, data| {
            let module_name = data
                .expect("Could not get module name target.")
                .get::<String>().unwrap();

            println!("export module {module_name}");
        }));
        self.add_action(&export_module_action);*/
    }
}
