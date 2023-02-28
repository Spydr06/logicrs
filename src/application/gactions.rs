use super::*;
use crate::{fatal::*, project::Project};

pub(super) type GActionCallbackFn = fn(Application, Option<&glib::Variant>);

pub(super) struct GAction<'a> {
    name: &'a str,
    accels: &'a[&'a str],
    parameter_type: Option<&'a glib::VariantTy>,
    callback: GActionCallbackFn
}

impl<'a> GAction<'a> {
    const fn new(name: &'a str, accels: &'a[&str], parameter_type: Option<&'a glib::VariantTy>, callback: GActionCallbackFn) -> Self {
        Self {
            name,
            accels,
            parameter_type,
            callback
        }
    }

    pub(super) fn name(&self) -> &str {
        self.name
    }

    pub(super) fn callback(&self) -> GActionCallbackFn {
        self.callback
    }

    pub(super) fn accels(&self) -> &[&str] {
        self.accels
    }
}

impl<'a> From<&GAction<'a>> for gio::SimpleAction {
    fn from(value: &GAction<'a>) -> Self {
        gio::SimpleAction::new(value.name, value.parameter_type)
    }
}

pub(super) const ACTIONS: &[GAction] = &[
    GAction::new("quit", &["<primary>Q", "<primary>W"], None, Application::gaction_quit),
    GAction::new("about", &["<primary>comma"], None, Application::gaction_about),  
    GAction::new("save", &["<primary>S"], None, Application::gaction_save),
    GAction::new("save-as", &["<primary><shift>S"], None, Application::gaction_save_as),
    GAction::new("open", &["<primary>O"], None, Application::gaction_open),
    GAction::new("new", &["<primary>N"], None, Application::gaction_new),
    GAction::new("delete-block", &["Delete"], None, Application::gaction_delete_block),
    GAction::new("create-new-module", &["<primary><shift>N"], None, Application::gaction_create_new_module),
    GAction::new("undo", &["<primary>Z"], None, Application::gaction_undo),
    GAction::new("redo", &["<primary>Y"], None, Application::gaction_redo),
    GAction::new("copy", &["<primary>C"], None, Application::gaction_copy),
    GAction::new("cut", &["<primary>X"], None, Application::gaction_cut),
    GAction::new("paste", &["<primary>V"], None, Application::gaction_paste),
    GAction::new("select-all", &["<primary>A"], None, Application::gaction_select_all),
    GAction::new("delete-module", &[], Some(glib::VariantTy::STRING), Application::gaction_delete_module),
    GAction::new("edit-module", &[], Some(glib::VariantTy::STRING), Application::gaction_edit_module),
    GAction::new("search-module", &["<primary>F"], None, Application::gaction_search_module),
    GAction::new("dark-theme", &[], None, Application::gaction_dark_mode),
    GAction::new("light-theme", &[], None, Application::gaction_light_mode),
    GAction::new("system-preference-theme", &[], None, Application::gaction_system_preference_mode)
];

impl Application {
    fn gaction_quit(self, _: Option<&glib::Variant>) {
        self.quit();
    }

    fn gaction_about(self, _: Option<&glib::Variant>) {
        self.show_about()
    }

    fn gaction_save(self, _: Option<&glib::Variant>) {
        if let Err(err) = self.imp().save(|_| ()) {
            let message =  format!("Error saving to '{}': {}", self.imp().file_name(), err);
            error!("{}", message);
            if let Some(window) = self.active_window() {
                dialogs::run(self, window, message, dialogs::basic_error);
            }
        }
    }

    fn gaction_save_as(self, _: Option<&glib::Variant>) {
        self.save_as(|_| ());
    }

    fn gaction_open(self, _: Option<&glib::Variant>) {
        self.open();
    }

    fn gaction_new(self, _: Option<&glib::Variant>) {
        self.open_new();
    }

    fn gaction_delete_block(self, _: Option<&glib::Variant>) {
        if let Some(plot_provider) = self.imp().current_plot() {
            let blocks = plot_provider.with_mut(|plot| 
                plot.selected().iter().map(|id| plot.get_block(*id).unwrap().to_owned()).collect()
            ).unwrap_or_default();
            self.new_action(Action::DeleteSelection(plot_provider, blocks, vec![]));
        }
    }

    fn gaction_create_new_module(self, _: Option<&glib::Variant>) {
        if let Some(window) = self.active_window() {
            dialogs::run(self, window, (), dialogs::new_module); 
        }
    }

    fn gaction_undo(self, _: Option<&glib::Variant>) {
        self.undo_action();
    }

    fn gaction_redo(self, _: Option<&glib::Variant>) {
        self.redo_action();
    }


    fn gaction_copy(self, _: Option<&glib::Variant>) {
        self.copy_clipboard(false);
    }

    fn gaction_cut(self, _: Option<&glib::Variant>) {
        self.copy_clipboard(true);
    }

    fn gaction_paste(self, _: Option<&glib::Variant>) {
        self.paste_clipboard();
    }

    fn gaction_select_all(self, _: Option<&glib::Variant>) {
        self.imp().with_current_plot_mut(|plot| plot.select_all());
        self.imp().rerender_editor();
    }

    fn gaction_delete_module(self, parameter: Option<&glib::Variant>) {
        let module_name = parameter
                .expect("Could not get module name target.")
                .get::<String>().unwrap();

        if let Some(window) = self.active_window() {
            dialogs::run(self, window, module_name, dialogs::confirm_delete_module);
        }
    }

    fn gaction_edit_module(self, parameter: Option<&glib::Variant>) {
        let module_name = parameter
            .expect("Could not get module name target.")
            .get::<String>().unwrap();
        self.imp().edit_module(module_name);
    }

    fn gaction_search_module(self, _: Option<&glib::Variant>) {
        self.imp().window()
            .borrow().as_ref().unwrap()
            .module_list().show_search();
    }

    fn gaction_dark_mode(self, _: Option<&glib::Variant>) {
        adw::StyleManager::default()
            .set_color_scheme(adw::ColorScheme::ForceDark)
    }

    fn gaction_light_mode(self, _: Option<&glib::Variant>) {
        adw::StyleManager::default()
            .set_color_scheme(adw::ColorScheme::ForceLight)
    }

    fn gaction_system_preference_mode(self, _: Option<&glib::Variant>) {
        adw::StyleManager::default()
            .set_color_scheme(adw::ColorScheme::Default)
    }

    pub(super) fn close_current_file<F>(&self, after: F)
    where
        F: Fn(&str) + 'static,
    {
        if !self.imp().is_dirty() {
            after("No");
            return;
        }

        let window = self.active_window().unwrap();
        let save_dialog = adw::MessageDialog::builder()
            .transient_for(&window)
            .modal(true)
            .heading("Save File?")
            .body(format!("There are unsaved changes in \"{}\". Do you want to save them?", self.imp().file_name()).as_str())
            .close_response("Cancel")
            .default_response("Yes")
            .build();

        save_dialog.add_response("Yes",  "Yes");
        save_dialog.add_response("Cancel", "Cancel");
        save_dialog.add_response("No", "No");
        save_dialog.set_response_enabled("Yes", true);
        save_dialog.set_response_appearance("Yes", adw::ResponseAppearance::Suggested);
        save_dialog.set_response_appearance("No", adw::ResponseAppearance::Destructive);
        save_dialog.present();

        save_dialog.connect_response(None, move |_, response| after(response));
    }

    pub(super) fn open_new(&self) {
        self.close_current_file(glib::clone!(@weak self as app => move |response| {
            match response {
                "Cancel" => return,
                "No" =>  {},
                "Yes" => {
                    if let Err(err) = app.imp().save(|_| ()) {
                        let message = format!("Error saving to '{}': {}", app.imp().file_name(), err);
                        error!("{}", message);
                        if let Some(window) = app.active_window() {
                            dialogs::run(app, window, message, dialogs::basic_error);
                        }
                        return;
                    }
                }
                _ => panic!("unexpected response \"{}\"", response)
            };

            app.imp().reset();
        }));
    }

    pub(crate) fn open(&self) {
        self.close_current_file(glib::clone!(@weak self as app => move |response| {
            match response {
                "Cancel" => return,
                "No" =>  {},
                "Yes" => {
                    if let Err(err) = app.imp().save(|_| ()) {
                        let message = format!("Error saving to '{}': {}", app.imp().file_name(), err);
                        error!("{}", message);
                        if let Some(window) = app.active_window() {
                            dialogs::run(app, window, message, dialogs::basic_error);
                        }
                        return;
                    }
                }
                _ => panic!("unexpected response \"{}\"", response)
            };

            let window = app.active_window().unwrap();

            let open_dialog = gtk::FileChooserNative::builder()
                .transient_for(&window)
                .modal(true)
                .title("Open File")
                .action(gtk::FileChooserAction::Open)
                .accept_label("Open")
                .cancel_label("Cancel")
                .filter(&Project::file_filter())
                .build();
            
            open_dialog.connect_response({
                let file_chooser = RefCell::new(Some(open_dialog.clone()));
                glib::clone!(@weak app => move |_, response| {
                    if let Some(file_chooser) = file_chooser.take() {
                        if response != gtk::ResponseType::Accept {
                            return;
                        }
                        for file in file_chooser.files().snapshot().into_iter() {
                            let file = file
                                .downcast()
                                .expect("unexpected type returned from file chooser");
                            match Project::load_from(&file) {
                                Ok(project) => app.imp().set_project(project, Some(file)),
                                _ => error!("Error opening file")
                            }
                        }
                    }
                    else {
                        warn!("got file chooser response after window was freed");
                    }
                })
            });
            
            open_dialog.show();
        }));
    }

    pub(super) fn save_as(&self, then: fn(&Application)) {
        let window = self.active_window().unwrap();

        let save_dialog = gtk::FileChooserNative::builder()
            .transient_for(&window)
            .modal(true)
            .title("Save As")
            .action(gtk::FileChooserAction::Save)
            .accept_label("Save")
            .filter(&Project::file_filter())
            .cancel_label("Cancel")
            .build();

        save_dialog.connect_response({
            let file_chooser = RefCell::new(Some(save_dialog.clone()));
            glib::clone!(@weak self as app => move |_, response| {
                if let Some(file_chooser) = file_chooser.take() {
                    if response != gtk::ResponseType::Accept {
                        return;
                    }
                    for file in file_chooser.files().snapshot().into_iter() {
                        let file: gio::File = file
                            .downcast()
                            .expect("unexpected type returned from file chooser");
                        if !file.query_exists(gio::Cancellable::NONE) {
                            file.create(gio::FileCreateFlags::NONE, gio::Cancellable::NONE).unwrap_or_die();
                        }
                        app.imp().set_file(file);
                        app.imp().save(then).unwrap_or_die();
                    }
                } else {
                    warn!("got file chooser response more than once");
                }
            })
        });

        save_dialog.show();
    }

    pub(super) fn show_about(&self) {
        let window = self.active_window().unwrap();
        let dialog = adw::AboutWindow::builder()
            .transient_for(&window)
            .modal(true)
            .application_name(config::APP_ID)
            .version(config::VERSION)
            .comments(config::DESCRIPTION)
            .copyright(config::COPYRIGHT)
            .developer_name("Spydr06")
            .developers(config::AUTHORS.split(':').map(|s| s.to_string()).collect())
            .website(config::REPOSITORY)
            .issue_url(&(config::REPOSITORY.to_owned() + "/issues"))
            .license_type(gtk::License::MitX11)
            .build();
        
        dialog.present();
    }
}