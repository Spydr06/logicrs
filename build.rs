use adw::gio::compile_resources;

fn main() {
    compile_resources(".", "logicrs.gresource.xml", "logicrs.gresource")
}
