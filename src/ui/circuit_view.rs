use std::{cell::RefCell, sync::Arc};

use gtk::{prelude::*, subclass::prelude::*, gio, glib, gdk};

use crate::{application::{data::*, selection::*}, renderer::*, simulator::{Connector, Connection, Linkage}};

glib::wrapper! {
    pub struct CircuitView(ObjectSubclass<CircuitViewTemplate>)
        @extends gtk::Box, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl CircuitView {
    pub fn new(app: ApplicationDataRef) -> Self {
        let circuit_view: Self = glib::Object::new::<Self>(&[]);
        circuit_view.imp().set_data(app).initialize();
        circuit_view
    }
}

#[derive(gtk::CompositeTemplate, Default)]
#[template(resource = "/content/circuit-view.ui")]
pub struct CircuitViewTemplate {
    #[template_child]
    drawing_area: TemplateChild<gtk::DrawingArea>,

    #[template_child]
    zoom_in: TemplateChild<gtk::Button>,

    #[template_child]
    zoom_out: TemplateChild<gtk::Button>,

    #[template_child]
    zoom_reset: TemplateChild<gtk::Button>,

    renderer: RefCell<Option<Arc<RefCell<CairoRenderer>>>>,
    data: RefCell<ApplicationDataRef>
}

impl CircuitViewTemplate {
    fn renderer(&self) -> Option<Arc<RefCell<CairoRenderer>>> {
        match self.renderer.borrow().as_ref() {
            Some(renderer) => Some(renderer.clone()),
            None => None
        }
    }

    fn set_data(&self, data: ApplicationDataRef) -> &Self {
        self.data.replace(data);
        self
    }

    fn setup_buttons(&self) {
        let renderer = self.renderer().unwrap();
        let r = renderer.clone();
        let w = self.drawing_area.to_owned();
        self.zoom_reset.connect_clicked(move |_| {
            r.borrow_mut().set_scale(DEFAULT_SCALE);
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

    fn init_dragging(&self) {
        let gesture_drag = gtk::GestureDrag::builder().button(gdk::ffi::GDK_BUTTON_PRIMARY as u32).build();
        let area = self.drawing_area.to_owned();
        let renderer = self.renderer().unwrap();
        let app_data = self.data.borrow().clone();
        gesture_drag.connect_drag_begin(move |gesture, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            drag_begin(&app_data, &area, renderer.borrow().scale(), x, y)
        });

        let area = self.drawing_area.to_owned();
        let renderer = self.renderer().unwrap();
        let app_data = self.data.borrow().clone();
        gesture_drag.connect_drag_update(move |gesture, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            drag_update(&app_data, &area, renderer.borrow().scale(), x, y)
        });

        let area = self.drawing_area.to_owned();
        let renderer = self.renderer().unwrap();
        let app_data = self.data.borrow().clone();
        gesture_drag.connect_drag_end(move |gesture, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            drag_end(&app_data, &area, renderer.borrow().scale(), x, y)
        });

        self.drawing_area.add_controller(&gesture_drag);
    }

    fn initialize(&self) {
        *self.renderer.borrow_mut() = Some(Arc::new(RefCell::new(CairoRenderer::new())));
        let renderer = self.renderer().unwrap();
        let app_data = self.data.borrow().clone();
        self.drawing_area.set_draw_func(move |area: &gtk::DrawingArea, context: &gtk::cairo::Context, width: i32, height: i32| {
            if let Err(err) = renderer.borrow_mut().callback(&app_data, area, context, width, height) {
                eprintln!("Error rendering CircuitView: {}", err);
                panic!();
            }
        });
        
        self.setup_buttons();
        self.init_dragging();
    }
}

#[glib::object_subclass]
impl ObjectSubclass for CircuitViewTemplate {
    const NAME: &'static str = "CircuitView";
    type Type = CircuitView;
    type ParentType = gtk::Box;

    fn class_init(my_class: &mut Self::Class) {
        Self::bind_template(my_class);
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}


impl ObjectImpl for CircuitViewTemplate {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

impl WidgetImpl for CircuitViewTemplate {
    fn realize(&self) {
        self.parent_realize();
    }

    fn unrealize(&self) {
        self.parent_unrealize();
    }
}

impl BoxImpl for CircuitViewTemplate {}

fn drag_begin(data: &ApplicationDataRef, area: &gtk::DrawingArea, scale: f64, x: f64, y: f64) {
    let position = ((x / scale) as i32, (y / scale) as i32);
    let mut data = data.lock().unwrap();

    data.unhighlight();
    
    match data.current_plot().get_block_at(position) {
        Some(index) => {
            if let Some(block) = data.current_plot_mut().get_block_mut(index) {
                if let Some(i) = block.position_on_connection(position, false) {
                    let start = block.get_connector_pos(Connector::Output(i));
                    data.set_selection(Selection::Connection {
                        block_id: index,
                        output: i,
                        start,
                        position: start
                    });
                }
                else {
                    block.set_start_pos(block.position());
                    block.set_highlighted(true);
                    data.set_selection(Selection::Single(index));
                }
            }
        }
        _ => {
            data.set_selection(Selection::Area(position, position));
        }
    }

    area.queue_draw();
}

fn drag_update(data: &ApplicationDataRef, area: &gtk::DrawingArea, scale: f64, x: f64, y: f64) {
    let position = ((x / scale) as i32, (y / scale) as i32);
    let mut data = data.lock().unwrap();

    match data.selection().clone() {
        Selection::Single(index) => {
            let block = data.current_plot_mut().get_block_mut(index).unwrap();
            let (start_x, start_y) = block.start_pos();
            block.set_position((start_x + position.0, start_y + position.1));
            area.queue_draw();
        }
        Selection::Connection { block_id, output, start, position: _ } => {
            data.set_selection(Selection::Connection { block_id, output, start, position: (start.0 + position.0, start.1 + position.1)});
            area.queue_draw();
        }
        Selection::Area(area_start, _) => {
            data.set_selection(Selection::Area(area_start, (area_start.0 + position.0, area_start.1 + position.1)));
            area.queue_draw();
        }
        _ => ()
    }
}

fn drag_end(data: &ApplicationDataRef, area: &gtk::DrawingArea, scale: f64, x: f64, y: f64) {
    if x == 0. && y == 0. {
        return;
    }

    let position = ((x / scale) as i32, (y / scale) as i32);
    let mut data = data.lock().unwrap();

    match data.selection().clone() { 
        Selection::Single(index) => {
            let block = data.current_plot_mut().get_block_mut(index).unwrap();
            let (start_x, start_y) = block.start_pos();
            block.set_position((start_x + position.0, start_y + position.1));
        },
        Selection::Connection { block_id, output, start: _, position } => {
            if let Some(block) = data.current_plot().get_block_at(position) {
                if let Some(i) = data.current_plot().get_block(block).unwrap().position_on_connection(position, true) {
                    data.current_plot_mut()
                        .get_block_mut(block_id)
                        .unwrap()
                        .add_connection(output, Connection::new(Linkage { block_id, port: output }, Linkage { block_id: block, port: i }));
                }
            }

            data.set_selection(Selection::None);
        }
        Selection::Area(_, _) => {
            data.highlight_area();
        }
        _ => {}
    }
    area.queue_draw()
}