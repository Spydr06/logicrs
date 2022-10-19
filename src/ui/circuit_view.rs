use std::{
    cell::RefCell,
    sync::Arc
};

use glib::{
    object_subclass,
    subclass::{
        object::{ObjectImpl, ObjectImplExt},
        types::ObjectSubclass,
        InitializingObject,
    },
    wrapper,
};

use gtk::{
    gio::{ActionGroup, ActionMap},
    prelude::{InitializingWidgetExt, DrawingAreaExtManual},
    subclass::{
        prelude::{WidgetImpl, DrawingAreaImpl},
        widget::{CompositeTemplate, WidgetImplExt},
    },
    Accessible, Buildable, CompositeTemplate, ConstraintTarget, Native, Root,
    ShortcutManager, Widget, DrawingArea,
};

use crate::{application::Application, renderer::{self, Renderer}};

wrapper! {
    pub struct CircuitView(ObjectSubclass<CircuitViewTemplate>)
        @extends DrawingArea, Widget,
        @implements ActionGroup, ActionMap, Accessible, Buildable, ConstraintTarget, Native, Root, ShortcutManager;
}

impl CircuitView {
    pub fn new(app: &Application) -> Self {
        glib::Object::new(&[("application", app)]).unwrap()
    }
}

#[derive(CompositeTemplate, Default)]
#[template(resource = "/app/circuit-view.ui")]
pub struct CircuitViewTemplate {
    renderer: RefCell<Option<Arc<Renderer>>>
}

impl CircuitViewTemplate {
    fn get_renderer(&self) -> Option<Arc<Renderer>> {
        match self.renderer.take() {
            Some(renderer) => Some(renderer.clone()),
            None => None
        }
    }
}

#[object_subclass]
impl ObjectSubclass for CircuitViewTemplate {
    const NAME: &'static str = "CircuitView";
    type Type = CircuitView;
    type ParentType = DrawingArea;

    fn class_init(my_class: &mut Self::Class) {
        Self::bind_template(my_class);
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for CircuitViewTemplate {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
    }
}

impl WidgetImpl for CircuitViewTemplate {
    fn realize(&self, widget: &Self::Type) {
        self.parent_realize(widget);

        *self.renderer.borrow_mut() = Some(Arc::new(Renderer::new()));

        let renderer = self.get_renderer().unwrap();
        widget.set_draw_func(move |area: &DrawingArea, context: &gtk::cairo::Context, width: i32, height: i32| {
            if let Err(err) = renderer.render_callback(area, context, width, height) {
                eprintln!("Error rendering CircuitView: {}", err);
                panic!();
            }
        });
    }

    fn unrealize(&self, widget: &Self::Type) {
        self.parent_unrealize(widget);
    }
}

impl DrawingAreaImpl for CircuitViewTemplate {}