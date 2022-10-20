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
    prelude::{InitializingWidgetExt, DrawingAreaExtManual, GestureDragExt},
    subclass::{
        prelude::{WidgetImpl, DrawingAreaImpl},
        widget::{CompositeTemplate, WidgetImplExt},
    },
    gdk,
    Accessible, Buildable, CompositeTemplate, ConstraintTarget, Native, Root,
    ShortcutManager, Widget, DrawingArea, GestureClick, GestureDrag, traits::{WidgetExt, GestureExt}
};

use crate::{application::Application, renderer::Renderer};

wrapper! {
    pub struct CircuitView(ObjectSubclass<CircuitViewTemplate>)
        @extends DrawingArea, Widget,
        @implements ActionGroup, ActionMap, Accessible, Buildable, ConstraintTarget, Native, Root, ShortcutManager, GestureClick, GestureDrag;
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

        {
            let renderer = self.get_renderer().unwrap();
            widget.set_draw_func(move |area: &DrawingArea, context: &gtk::cairo::Context, width: i32, height: i32| {
                if let Err(err) = renderer.render_callback(area, context, width, height) {
                    eprintln!("Error rendering CircuitView: {}", err);
                    panic!();
                }
            });
        }

        {
            let gesture_drag = GestureDrag::builder().button(gdk::ffi::GDK_BUTTON_PRIMARY as u32).build();

            let w = widget.to_owned();
            gesture_drag.connect_drag_begin(move |gesture, x, y| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                crate::APPLICATION_DATA.with(|data| {
                    let mut data = data.borrow_mut();
                    match data.get_block_at((x as i32, y as i32)) {
                        Some(index) => {
                            data.highlight(index);
                            if let Some(block) = data.get_block_mut(index) {
                                block.set_start_pos(block.position());
                            }
                        }
                        None => data.unhighlight()
                    }

                    w.queue_draw();
                });
            });

            let w = widget.to_owned();
            gesture_drag.connect_drag_update(move |gesture, x, y| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                crate::APPLICATION_DATA.with(|data| {
                    let mut data = data.borrow_mut();
                    if let Some(block) = data.get_highlighted_mut() {
                        let (start_x, start_y) = block.start_pos();
                        block.set_position((start_x + x as i32, start_y + y as i32));
                        w.queue_draw();
                    }
                });
            });

            let w = widget.to_owned();
            gesture_drag.connect_drag_end(move |gesture, x, y| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                if x == 0. && y == 0. {
                    return;
                }

                crate::APPLICATION_DATA.with(|data| {
                    let mut data = data.borrow_mut();
                    if let Some(block) = data.get_highlighted_mut() {
                        let (start_x, start_y) = block.start_pos();
                        block.set_position((start_x + x as i32, start_y + y as i32));
                       w.queue_draw();
                    }
                });
            });

            widget.add_controller(&gesture_drag);
        }

        
    }

    fn unrealize(&self, widget: &Self::Type) {
        self.parent_unrealize(widget);
    }
}

impl DrawingAreaImpl for CircuitViewTemplate {}