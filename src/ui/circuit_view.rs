use std::{cell::RefCell, sync::Arc};

use gtk::{prelude::*, subclass::prelude::*, gio, glib, gdk};

use crate::{selection::*, renderer::*, simulator::*};

glib::wrapper! {
    pub struct CircuitView(ObjectSubclass<CircuitViewTemplate>)
        @extends gtk::Box, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl CircuitView {
    pub fn new(plot_provider: PlotProvider) -> Self {
        let circuit_view: Self = glib::Object::new::<Self>(&[]);
        circuit_view.imp().set_plot_provider(plot_provider).initialize();
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

    #[template_child]
    context_menu: TemplateChild<gtk::PopoverMenu>,

    renderer: RefCell<Option<Arc<RefCell<CairoRenderer>>>>,
    plot_provider: RefCell<PlotProvider>,
}

impl CircuitViewTemplate {
    pub fn rerender(&self) {
        self.drawing_area.queue_draw();
    }

    fn renderer(&self) -> Option<Arc<RefCell<CairoRenderer>>> {
        match self.renderer.borrow().as_ref() {
            Some(renderer) => Some(renderer.clone()),
            None => None
        }
    }

    pub fn plot_provider(&self) -> PlotProvider {
        self.plot_provider.borrow().clone()
    }

    fn set_plot_provider(&self, plot_provider: PlotProvider) -> &Self {
        self.plot_provider.replace(plot_provider);
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

        let original_provider = self.plot_provider.borrow();

        let provider = original_provider.clone();
        gesture_drag.connect_drag_begin(move |gesture, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            provider.with_mut(|plot| drag_begin(plot, &area, renderer.borrow().scale(), x, y));
        });

        let area = self.drawing_area.to_owned();
        let renderer = self.renderer().unwrap();
        let provider = original_provider.clone();
        gesture_drag.connect_drag_update(move |gesture, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            provider.with_mut(|plot| drag_update(plot, &area, renderer.borrow().scale(), x, y));
        });

        let area = self.drawing_area.to_owned();
        let renderer = self.renderer().unwrap();
        let provider = original_provider.clone();
        gesture_drag.connect_drag_end(move |gesture, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            provider.with_mut(|plot| drag_end(plot, &area, renderer.borrow().scale(), x, y));
        });

        self.drawing_area.add_controller(&gesture_drag);
    }

    fn init_context_menu(&self) {
        let gesture = gtk::GestureClick::builder().button(gdk::ffi::GDK_BUTTON_SECONDARY as u32).build();
        gesture.connect_pressed(glib::clone!(@weak self as widget => move |gesture, _, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            widget.context_menu(x, y);
        }));
        self.drawing_area.add_controller(&gesture);
    }

    fn initialize(&self) {
        *self.renderer.borrow_mut() = Some(Arc::new(RefCell::new(CairoRenderer::new())));
        let renderer = self.renderer().unwrap();
        let provider = self.plot_provider.borrow().clone();
        self.drawing_area.set_draw_func(move |area: &gtk::DrawingArea, context: &gtk::cairo::Context, width: i32, height: i32| {
            provider.with_mut(|plot| {
                if let Err(err) = renderer.borrow_mut().callback(plot, area, context, width, height) {
                    eprintln!("Error rendering CircuitView: {}", err);
                    panic!();
                }
            });
        });
        
        self.setup_buttons();
        self.init_dragging();
        self.init_context_menu();
    }

    fn context_menu(&self, x: f64, y: f64) {        
        let scale = if let Some(r) = self.renderer.borrow().as_ref() { r.borrow().scale() } else { 1.0 };
        let position = ((x / scale) as i32, (y / scale) as i32);

        self.plot_provider.borrow_mut().with_mut(|plot| {
            match plot.get_block_at(position) {
                Some(index) => {
                    if let Some(block) = plot.get_block_mut(index) {
                        block.set_start_pos(block.position());
                        block.set_highlighted(true);

                        if let Selection::None = plot.selection() {
                            plot.unhighlight();
                            plot.set_selection(Selection::Single(index));
                        }

                        drop(plot);

                        self.context_menu.set_pointing_to(Some(&gdk::Rectangle::new(position.0, position.1, 1, 1)));
                        self.context_menu.popup();
                    }
                }
                None => {}
            }
        });

        self.drawing_area.queue_draw();
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

fn drag_begin(plot: &mut Plot, area: &gtk::DrawingArea, scale: f64, x: f64, y: f64) {
    let position = ((x / scale) as i32, (y / scale) as i32);

    plot.unhighlight();
    
    match plot.get_block_at(position) {
        Some(index) => {
            if let Some(block) = plot.get_block_mut(index) {
                if let Some(i) = block.position_on_connection(position, false) {
                    let start = block.get_connector_pos(Connector::Output(i));
                    plot.set_selection(Selection::Connection {
                        block_id: index,
                        output: i,
                        start,
                        position: start
                    });
                }
                else {
                    block.set_start_pos(block.position());
                    block.set_highlighted(true);
                    plot.set_selection(Selection::Single(index));
                }
            }
        }
        _ => {
            plot.set_selection(Selection::Area(position, position));
        }
    }

    area.queue_draw();
}

fn drag_update(plot: &mut Plot, area: &gtk::DrawingArea, scale: f64, x: f64, y: f64) {
    let position = ((x / scale) as i32, (y / scale) as i32);

    match plot.selection().clone() {
        Selection::Single(index) => {
            let block = plot.get_block_mut(index).unwrap();
            let (start_x, start_y) = block.start_pos();
            block.set_position((start_x + position.0, start_y + position.1));
            area.queue_draw();
        }
        Selection::Connection { block_id, output, start, position: _ } => {
            plot.set_selection(Selection::Connection { block_id, output, start, position: (start.0 + position.0, start.1 + position.1)});
            area.queue_draw();
        }
        Selection::Area(area_start, _) => {
            plot.set_selection(Selection::Area(area_start, (area_start.0 + position.0, area_start.1 + position.1)));
            area.queue_draw();
        }
        _ => ()
    }
}

fn drag_end(plot: &mut Plot, area: &gtk::DrawingArea, scale: f64, x: f64, y: f64) {
    if x == 0. && y == 0. {
        return;
    }

    let position = ((x / scale) as i32, (y / scale) as i32);

    match plot.selection().clone() { 
        Selection::Single(index) => {
            let block = plot.get_block_mut(index).unwrap();
            let (start_x, start_y) = block.start_pos();
            block.set_position((start_x + position.0, start_y + position.1));
        },
        Selection::Connection { block_id, output, start: _, position } => {
            if let Some(block) = plot.get_block_at(position) {
                if let Some(i) = plot.get_block(block).unwrap().position_on_connection(position, true) {
                    plot.get_block_mut(block_id)
                        .unwrap()
                        .add_connection(output, Connection::new(Linkage { block_id, port: output }, Linkage { block_id: block, port: i }));
                }
            }

            plot.set_selection(Selection::None);
        }
        Selection::Area(_, _) => {
            plot.highlight_area();
        }
        _ => {}
    }
    area.queue_draw()
}
