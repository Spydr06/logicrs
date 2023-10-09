pub mod action;
pub mod clipboard;
pub mod editor;
pub mod gactions;
pub mod selection;
pub mod template;
pub mod user_settings;

use crate::application::gactions::Theme;
use crate::application::user_settings::UserSettingsKey::ThemeKey;
use crate::application::user_settings::UserSettingsValue::ThemeValue;
use crate::{application::clipboard::Clipboard, config, ui::dialogs};
use action::*;
use adw::traits::MessageDialogExt;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use selection::SelectionField;
use std::cell::RefCell;

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
        gio::resources_register_include!("logicrs.gresource")
            .expect("Failed to register resources.");

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
            Clipboard::Blocks(..) => {
                let position = self
                    .imp()
                    .current_circuit_view()
                    .map(|view| view.mouse_world_position())
                    .unwrap_or_default();

                match clipboard.paste_to(self.imp().current_plot().unwrap(), position) {
                    Ok(action) => self.new_action(action),
                    Err(err) => dialogs::run(
                        self.to_owned(),
                        self.active_window().unwrap(),
                        err,
                        dialogs::basic_error,
                    ),
                }
            }
            Clipboard::Module(_) => todo!(),
            Clipboard::Empty => {}
        }
    }

    pub fn paste_clipboard(&self) {
        let display = RootExt::display(&self.active_window().unwrap());
        display.clipboard().read_text_async(
            None as Option<&gio::Cancellable>,
            glib::clone!(@weak self as app => move |pasted| {
                match pasted
                    .map_err(|err| err.to_string())
                    .and_then(|text| text.ok_or(String::new()))
                    .and_then(|text| Clipboard::deserialize(text.as_str()))
                {
                    Ok(clipboard) => app.apply_clipboard(clipboard),
                    Err(err) => warn!("Error pasting from clipboard: {err}")
                }
            }),
        );
    }

    pub fn cut_clipboard(&self, clipboard: Clipboard) {
        match clipboard {
            Clipboard::Blocks(blocks, connections) => self.new_action(Action::DeleteSelection(
                self.imp().current_plot().unwrap(),
                blocks,
                connections,
                vec![],
            )),
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
            Err(err) => warn!("Error serializing clipboard: {err}"),
        }
    }

    pub fn quit(&self) {
        self.close_current_file(glib::clone!(@weak self as app => move |response| {
            match response {
                "Cancel" => {
                },
                "No" =>  {
                    app.imp().shutdown();
                },
                "Yes" => {
                    if let Err(err) = app.imp().save(|app| app.imp().shutdown()) {
                        let message = format!("Error saving to '{}': {}", app.imp().file_name(), err);
                        error!("{}", message);
                        if let Some(window) = app.active_window() {
                            dialogs::run(app, window, message, dialogs::basic_error);
                        }
                    }
                }
                _ => panic!("unexpected response \"{}\"", response)
            }
        }));
    }

    pub(self) fn setup_gactions(&self) {
        gactions::ACTIONS.iter().for_each(|gaction| {
            let mut accels = gaction.accels().to_vec();
            let temp: Vec<String>;
            if cfg!(target_os = "macos") {
                temp = gaction
                    .accels()
                    .iter()
                    .map(|s| s.replace("<primary>", "<meta>"))
                    .collect::<Vec<String>>();
                accels = temp.iter().map(|x| &**x).collect::<Vec<&str>>();
            }
            let callback = gaction.callback();
            let action = gio::SimpleAction::from(gaction);

            if gaction.name() == "change-theme" {
                let theme_variant = match self.imp().user_settings().borrow().get_setting(ThemeKey)
                {
                    Some(ThemeValue(custom_theme)) => custom_theme.to_variant(),
                    None => Theme::SystemPreference.to_variant(),
                };

                action.set_state(&theme_variant);
            }

            action.connect_activate(glib::clone!(
                @weak self as app => move |action, parameter| callback(app, action, parameter)
            ));
            self.add_action(&action);
            self.set_accels_for_action(&format!("app.{}", gaction.name()), &accels);
        });
    }
}
