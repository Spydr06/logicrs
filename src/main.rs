mod application;
mod renderer;
mod ui;

use application::info::Info;

fn main() {
    unsafe { renderer::prelude::load_gl_pointers() };

    let info = Info::new().title("Logic Rs").default_size(1366, 768);

    let mut application = application::Application::new(info);
    application.run();
}
