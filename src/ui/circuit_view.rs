use std::{cell::RefCell, sync::Arc};

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
    prelude::{InitializingWidgetExt, DrawingAreaExtManual, GestureDragExt, ButtonExt},
    subclass::{
        prelude::{WidgetImpl, BoxImpl},
        widget::{CompositeTemplate, WidgetImplExt, WidgetClassSubclassExt},
    },
    gdk,
    Accessible, Buildable, CompositeTemplate, ConstraintTarget, Native, Root,
    ShortcutManager, Widget, DrawingArea, GestureDrag, traits::{WidgetExt, GestureExt},
    Box, TemplateChild, Button
};

use crate::{
    application::{
        Application,
        data::Selection
    }, 
    renderer::{
        self,
        Renderer, CairoRenderer
    }
};

wrapper! {
    pub struct CircuitView(ObjectSubclass<CircuitViewTemplate>)
        @extends Box, Widget,
        @implements ActionGroup, ActionMap, Accessible, Buildable, ConstraintTarget, Native, Root, ShortcutManager;
}

impl CircuitView {
    pub fn new(app: &Application) -> Self {
        glib::Object::new(&[("application", app)]).unwrap()
    }
}

#[derive(CompositeTemplate, Default)]
#[template(resource = "/content/circuit-view.ui")]
pub struct CircuitViewTemplate {
    #[template_child]
    drawing_area: TemplateChild<DrawingArea>,

    #[template_child]
    zoom_in: TemplateChild<Button>,

    #[template_child]
    zoom_out: TemplateChild<Button>,

    #[template_child]
    zoom_reset: TemplateChild<Button>,

    renderer: RefCell<Option<Arc<RefCell<CairoRenderer>>>>,
}

impl CircuitViewTemplate {
    fn renderer(&self) -> Option<Arc<RefCell<CairoRenderer>>> {
        match self.renderer.borrow().as_ref() {
            Some(renderer) => Some(renderer.clone()),
            None => None
        }
    }

    fn setup_buttons(&self) {
        let renderer = self.renderer().unwrap();
        let r = renderer.clone();
        let w = self.drawing_area.to_owned();
        self.zoom_reset.connect_clicked(move |_| {
            r.borrow_mut().set_scale(renderer::DEFAULT_SCALE);
            w.queue_draw();
            //println!("scale: {}%", r.lock().unwrap().scale() * 100.);
        });
        let r = renderer.clone();
        let w = self.drawing_area.to_owned();
        self.zoom_in.connect_clicked(move |_| {
            let mut r = r.borrow_mut();
            let scale = r.scale();
            r.set_scale(scale * 1.1);
            w.queue_draw();
            //println!("scale: {}%", scale * 100.);
        });
        let r = renderer.clone();
        let w = self.drawing_area.to_owned();
        self.zoom_out.connect_clicked(move |_| {
            let mut r = r.borrow_mut();
            let scale = r.scale();
            r.set_scale(scale / 1.1);
            w.queue_draw();
            //println!("scale: {}%", scale * 100.);
        });
    }
}

#[object_subclass]
impl ObjectSubclass for CircuitViewTemplate {
    const NAME: &'static str = "CircuitView";
    type Type = CircuitView;
    type ParentType = Box;

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

        *self.renderer.borrow_mut() = Some(Arc::new(RefCell::new(CairoRenderer::new())));
        self.setup_buttons();

        let renderer = self.renderer().unwrap();
        self.drawing_area.set_draw_func(move |area: &DrawingArea, context: &gtk::cairo::Context, width: i32, height: i32| {
            if let Err(err) = renderer.borrow_mut().callback(area, context, width, height) {
                eprintln!("Error rendering CircuitView: {}", err);
                panic!();
            }
        });

        let gesture_drag = GestureDrag::builder().button(gdk::ffi::GDK_BUTTON_PRIMARY as u32).build();
        let area = self.drawing_area.to_owned();
        let renderer = self.renderer().unwrap();
        gesture_drag.connect_drag_begin(move |gesture, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            drag_begin(&area, renderer.borrow().scale(), x, y)
        });

        let area = self.drawing_area.to_owned();
        let renderer = self.renderer().unwrap();
        gesture_drag.connect_drag_update(move |gesture, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            drag_update(&area, renderer.borrow().scale(), x, y)
        });

        let area = self.drawing_area.to_owned();
        let renderer = self.renderer().unwrap();
        gesture_drag.connect_drag_end(move |gesture, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            drag_end(&area, renderer.borrow().scale(), x, y)
        });

        self.drawing_area.add_controller(&gesture_drag);
    }

    fn unrealize(&self, widget: &Self::Type) {
        self.parent_unrealize(widget);
    }
}

impl BoxImpl for CircuitViewTemplate {}

fn drag_begin(area: &DrawingArea, scale: f64, x: f64, y: f64) {
    crate::APPLICATION_DATA.with(|data| {
        let mut data = data.borrow_mut();
        let position = ((x / scale) as i32, (y / scale) as i32);
    
        data.unhighlight();
        
        match data.current_plot().get_block_at(position) {
            Some(index) => {
                if let Some(block) = data.current_plot_mut().get_block_mut(index) {
                    block.set_start_pos(block.position());
                    block.set_highlighted(true);
                }
                data.set_selection(Selection::Single(index));
            }
            _ => {
                data.set_selection(Selection::Area(position, position));
            }
        }
        
        area.queue_draw();
    });
}

fn drag_update(area: &DrawingArea, scale: f64, x: f64, y: f64) {
    crate::APPLICATION_DATA.with(|data| {
        let mut data = data.borrow_mut();
        let position = ((x / scale) as i32, (y / scale) as i32);

        match data.selection().clone() {
            Selection::Single(index) => {
                let block = data.current_plot_mut().get_block_mut(index).unwrap();
                let (start_x, start_y) = block.start_pos();
                block.set_position((start_x + position.0, start_y + position.1));
                area.queue_draw();
            }
            Selection::Area(area_start, _) => {
                data.set_selection(Selection::Area(area_start, (area_start.0 + position.0, area_start.1 + position.1)));
                area.queue_draw();
            }
            _ => ()
        }
    });
}

fn drag_end(area: &DrawingArea, scale: f64, x: f64, y: f64) {
    if x == 0. && y == 0. {
        return;
    }

    crate::APPLICATION_DATA.with(|data| {
        let mut data = data.borrow_mut();
        let position = ((x / scale) as i32, (y / scale) as i32);

        match data.selection().clone() { 
            Selection::Single(index) => {
                let block = data.current_plot_mut().get_block_mut(index).unwrap();
                let (start_x, start_y) = block.start_pos();
                block.set_position((start_x + position.0, start_y + position.1));
            },
            Selection::Area(_, _) => {
                data.highlight_area();
            }
            _ => {}
        }

        area.queue_draw()
    });
}