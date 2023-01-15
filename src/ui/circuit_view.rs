use std::cell::RefCell;
use gtk::{prelude::*, subclass::prelude::*, gio, glib, gdk};
use crate::{selection::*, renderer::*, simulator::*, fatal::FatalResult};

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

    #[template_child]
    area_context_menu: TemplateChild<gtk::PopoverMenu>,

    #[template_child]
    left_osd_box: TemplateChild<gtk::Box>,

    #[template_child]
    left_osd_label: TemplateChild<gtk::Label>,

    renderer: RefCell<CairoRenderer>,
    plot_provider: RefCell<PlotProvider>,

    ctrl_down: RefCell<bool>
}

impl CircuitViewTemplate {
    pub fn rerender(&self) {
        self.drawing_area.queue_draw();
    }

    pub fn plot_provider(&self) -> PlotProvider {
        self.plot_provider.borrow().clone()
    }

    fn set_plot_provider(&self, plot_provider: PlotProvider) -> &Self {
        self.plot_provider.replace(plot_provider);
        self
    }

    fn init_buttons(&self) {
        self.zoom_reset.connect_clicked(glib::clone!(@weak self as widget => move |_| {
            let mut r = widget.renderer.borrow_mut();
            r.set_scale(DEFAULT_SCALE);
            r.translate((0., 0.));
            widget.drawing_area.queue_draw();
            //println!("scale: {}%", r.lock().unwrap().scale() * 100.);
        }));

        self.zoom_in.connect_clicked(glib::clone!(@weak self as widget => move |_| {
            let mut r = widget.renderer.borrow_mut();
            r.zoom(1.1);
            widget.drawing_area.queue_draw();
            //println!("scale: {}%", scale * 100.);
        }));

        self.zoom_out.connect_clicked(glib::clone!(@weak self as widget => move |_| {
            let mut r = widget.renderer.borrow_mut();
            r.zoom(0.9);
            widget.drawing_area.queue_draw();
            //println!("scale: {}%", scale * 100.);
        }));
    }

    fn init_dragging(&self) {
        let gesture_drag = gtk::GestureDrag::builder().button(gdk::ffi::GDK_BUTTON_PRIMARY as u32).build();
        gesture_drag.connect_drag_begin(glib::clone!(@weak self as widget => move |gesture, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            widget.drawing_area.grab_focus();
            
            if *widget.ctrl_down.borrow() {
                widget.renderer.borrow_mut().save_translation();
                widget.set_left_osd_visible(true);
            }
            else {
                widget.plot_provider.borrow().with_mut(|plot| drag_begin(plot, &widget.drawing_area, widget.renderer.borrow().world_coords(x, y)));
            }

        }));

        gesture_drag.connect_drag_update(glib::clone!(@weak self as widget => move |gesture, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            let scale = widget.renderer.borrow().scale();

            if *widget.ctrl_down.borrow() {
                let original_translation = widget.renderer.borrow().original_translation();
                widget.renderer.borrow_mut().translate((x / scale + original_translation.0, y / scale + original_translation.1));
                widget.drawing_area.queue_draw();
                let translation = widget.renderer.borrow().translation();
                widget.set_left_osd_label(&format!("{}, {}", translation.0 as i32, translation.1 as i32));
            }
            else {
                widget.plot_provider.borrow().with_mut(|plot| drag_update(plot, &widget.drawing_area, ((x / scale) as i32, (y / scale) as i32)));
            }

        }));

        gesture_drag.connect_drag_end(glib::clone!(@weak self as widget => move |gesture, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);

            if *widget.ctrl_down.borrow() && (x != 0. || y != 0.) {
                widget.set_left_osd_visible(false);
            }
            else {
                let scale = widget.renderer.borrow().scale();
                widget.plot_provider.borrow().with_mut(|plot| drag_end(plot, &widget.drawing_area, ((x / scale) as i32, (y / scale) as i32)));
            }
        }));

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

    fn init_keyboard(&self) {
        let key_controller = gtk::EventControllerKey::new();
        key_controller.connect_key_pressed(glib::clone!(@weak self as widget => @default-panic, move |_, key, _, _| {
            if key == gdk::Key::Control_L {
                widget.ctrl_down.replace(true);
            }
            gtk::Inhibit(true)
        }));

        key_controller.connect_key_released(glib::clone!(@weak self as widget => @default-panic, move |_, key, _, _| 
            if key == gdk::Key::Control_L {
                widget.ctrl_down.replace(false);
                widget.set_left_osd_visible(false);
            }
        ));
        self.drawing_area.add_controller(&key_controller);
    }

    fn init_scrolling(&self) {
        let scroll_controller = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        scroll_controller.connect_scroll(glib::clone!(@weak self as widget => @default-panic, move |_, _, y| {
            widget.renderer.borrow_mut().zoom(if y > 0. { 0.9 } else { 1.1 });
            widget.drawing_area.queue_draw();

            gtk::Inhibit(true)
        }));
        self.drawing_area.add_controller(&scroll_controller);
    }

    fn initialize(&self) {
        self.drawing_area.set_draw_func(glib::clone!(@weak self as widget => move |area, context, width, height|
            widget.plot_provider.borrow().with_mut(|plot| widget.renderer.borrow_mut().callback(plot, area, context, width, height).unwrap_or_die());
        ));

        self.drawing_area.set_focusable(true);
        self.drawing_area.grab_focus();
        self.drawing_area.set_focus_on_click(true);

        self.init_buttons();
        self.init_dragging();
        self.init_keyboard();
        self.init_scrolling();
        self.init_context_menu();
    }

    fn context_menu(&self, x: f64, y: f64) {        
        let scale = self.renderer.borrow().scale();
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
                None => {
                    self.area_context_menu.set_pointing_to(Some(&gdk::Rectangle::new(position.0, position.1, 1, 1)));
                    self.area_context_menu.popup();
                }
            }
        });

        self.drawing_area.queue_draw();
    }

    fn set_left_osd_visible(&self, visible: bool) {
        self.left_osd_box.set_visible(visible);
    }

    fn set_left_osd_label<'a>(&self, label: &'a str) {
        self.left_osd_label.set_text(label);
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

fn drag_begin(plot: &mut Plot, area: &gtk::DrawingArea, position: (i32, i32)) {
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

fn drag_update(plot: &mut Plot, area: &gtk::DrawingArea, offset: (i32, i32)) {
    match plot.selection().clone() {
        Selection::Single(index) => {
            let block = plot.get_block_mut(index).unwrap();
            let (start_x, start_y) = block.start_pos();
            block.set_position((start_x + offset.0, start_y + offset.1));
            area.queue_draw();
        }
        Selection::Connection { block_id, output, start, position: _ } => {
            plot.set_selection(Selection::Connection { block_id, output, start, position: (start.0 + offset.0, start.1 + offset.1)});
            area.queue_draw();
        }
        Selection::Area(area_start, _) => {
            plot.set_selection(Selection::Area(area_start, (area_start.0 + offset.0, area_start.1 + offset.1)));
            area.queue_draw();
        }
        _ => ()
    }
}

fn drag_end(plot: &mut Plot, area: &gtk::DrawingArea, offset: (i32, i32)) {
    match plot.selection().clone() { 
        Selection::Single(index) => {
            let block = plot.get_block_mut(index).unwrap();
            let (start_x, start_y) = block.start_pos();
            block.set_position((start_x + offset.0, start_y + offset.1));
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
