mod application;
mod ui;

use application::info::Info;

fn main() -> Result<(), ()> {
    /*let application = Application::builder()
        .application_id("com.spydr06.LogicRs")
        .build();

    application.connect_activate(|app| {
        // ActionRows are only available in Adwaita
        let row = ActionRow::builder()
            .activatable(true)
            .title("Click me")
            .build();
        row.connect_activated(|_| {
            eprintln!("Clicked!");
        });

        let list = ListBox::builder()
            .margin_top(32)
            .margin_end(32)
            .margin_bottom(32)
            .margin_start(32)
            .selection_mode(SelectionMode::None)
            // makes the list look nicer
            .css_classes(vec![String::from("boxed-list")])
            .build();
        list.append(&row);

        // Combine the content in a box
        let content = Box::new(Orientation::Vertical, 0);
        // Adwaitas' ApplicationWindow does not include a HeaderBar
        content.append(&HeaderBar::new());
        content.append(&list);

        let style = StyleManager::default();
        style.set_color_scheme(ColorScheme::ForceDark);

        let window = ApplicationWindow::builder()
            .application(app)
            .title(TITLE)
            .default_width(350)
            // add content to window
            .content(&content)
            .decorated(false)
            .default_width(DEFAULT_WIDTH)
            .default_height(DEFAULT_HEIGHT)
            .build();
        window.show();
    });

    application.run();*/

    let info = Info::new().title("Logic Rs").default_size(1366, 768);

    let mut application = application::Application::new(info);
    application.run()
}
