use adw::{prelude::*, ColorScheme, StyleManager};

use main_window::MainWindow;

mod circuit_view;
mod main_window;
mod module_list;
mod template;

pub fn build_ui(app: &adw::Application) {
    StyleManager::default().set_color_scheme(ColorScheme::ForceDark);

    let window = MainWindow::new(app);
    window.show();
}
