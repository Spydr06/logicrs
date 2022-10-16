mod application;
mod renderer;
mod ui;

use adw::prelude::ApplicationExtManual;
use application::Application;

fn main() {
    unsafe {
        renderer::prelude::load_gl_pointers()
    };

    let application = Application::new();
    std::process::exit(application.run());
}
