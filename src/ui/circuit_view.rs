use std::cell::{RefCell, Cell};
use gtk::{prelude::*, subclass::prelude::*, gio, glib, gdk};
use crate::{renderer::{*, vector::*}, simulator::*, fatal::FatalResult, application::{selection::*, Application, action::Action, editor::{EditorMode, GRID_SIZE}}};

glib::wrapper! {
    pub struct CircuitView(ObjectSubclass<CircuitViewTemplate>)
        @extends gtk::Box, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl CircuitView {
    pub fn new(app: Application, plot_provider: PlotProvider) -> Self {
        let circuit_view: Self = glib::Object::new::<Self>(&[]);
        circuit_view.imp()
            .set_application(app)
            .set_plot_provider(plot_provider)
            .initialize();
        circuit_view
    }

    pub fn focused(&self) -> bool {
        self.imp().drawing_area.has_focus()
    }

    pub fn set_editor_mode(&self, editor_mode: EditorMode) {
        self.imp().editor_mode.replace(editor_mode);
    }

    pub fn rerender(&self) {
        self.imp().rerender();
    }


    pub fn plot_provider(&self) -> PlotProvider {
        self.imp().plot_provider()
    }

    pub fn mouse_world_position(&self) -> Vector2<f64> {
        let mouse_position = self.imp().mouse_position.get();
        self.imp().renderer.borrow().screen_to_world(mouse_position)
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
    ctrl_down: Cell<bool>,
    shift_down: Cell<bool>,
    application: RefCell<Application>,
    editor_mode: RefCell<EditorMode>,
    mouse_position: Cell<Vector2<f64>>
}

impl CircuitViewTemplate {
    fn rerender(&self) {
        self.drawing_area.queue_draw();
    }

    fn plot_provider(&self) -> PlotProvider {
        self.plot_provider.borrow().clone()
    }

    fn set_plot_provider(&self, plot_provider: PlotProvider) -> &Self {
        self.plot_provider.replace(plot_provider);
        self
    }

    fn set_application(&self, app: Application) -> &Self {
        self.application.replace(app);
        self
    }

    fn init_buttons(&self) {
        self.zoom_reset.connect_clicked(glib::clone!(@weak self as widget => move |_| {
            let mut r = widget.renderer.borrow_mut();
            r.set_scale(DEFAULT_SCALE);
            r.translate(Vector2::default());
            widget.drawing_area.queue_draw();
            widget.left_osd_label.set_label("0, 0");
        }));

        self.zoom_in.connect_clicked(glib::clone!(@weak self as widget => move |_| {
            let mut r = widget.renderer.borrow_mut();
            r.zoom(1.1, None);
            widget.drawing_area.queue_draw();
        }));

        self.zoom_out.connect_clicked(glib::clone!(@weak self as widget => move |_| {
            let mut r = widget.renderer.borrow_mut();
            r.zoom(0.9, None);
            widget.drawing_area.queue_draw();
        }));
    }

    fn init_mouse(&self) {
        let mouse_controller = gtk::EventControllerMotion::new();
        mouse_controller.connect_motion(glib::clone!(@weak self as widget => move |_, x, y| widget.on_mouse_move(x, y)));
        self.drawing_area.add_controller(&mouse_controller);

        let gesture_drag = gtk::GestureDrag::builder().button(gdk::ffi::GDK_BUTTON_PRIMARY as u32).build();
        gesture_drag.connect_drag_begin(glib::clone!(@weak self as widget => move |gesture, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            widget.drawing_area.grab_focus();
            
            if widget.ctrl_down.get() {
                widget.renderer.borrow_mut().save_translation();
                widget.set_left_osd_visible(true);
            }
            else {
                widget.drag_begin(VectorCast::cast(widget.renderer.borrow().screen_to_world(Vector2(x, y))));
            }
        }));

        gesture_drag.connect_drag_update(glib::clone!(@weak self as widget => move |gesture, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            let scale = widget.renderer.borrow().scale();

            if widget.ctrl_down.get() {
                let original_translation = widget.renderer.borrow().original_translation();
                widget.renderer.borrow_mut().translate((x + original_translation.x(), y + original_translation.y()).into());
                widget.drawing_area.queue_draw();
                let translation = widget.renderer.borrow().translation();
                widget.set_left_osd_label(&format!("{}, {}", translation.x() as i32, translation.y() as i32));
            }
            else {
                widget.drag_update(Vector2((x / scale) as i32, (y / scale) as i32));
            }
        }));

        let app = &*self.application.borrow();
        gesture_drag.connect_drag_end(glib::clone!(@weak self as widget, @weak app => move |gesture, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);

            if widget.ctrl_down.get() && (x != 0. || y != 0.) {
                widget.set_left_osd_visible(false);
            }
            else {
                let scale = widget.renderer.borrow().scale();
                widget.drag_end(Vector2((x / scale) as i32, (y / scale) as i32));
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
            match key {
                gdk::Key::Control_L | gdk::Key::Control_R => widget.ctrl_down.set(true),
                gdk::Key::Shift_L | gdk::Key::Shift_R => widget.shift_down.set(true),
                _ => ()
            }
            gtk::Inhibit(true)
        }));

        key_controller.connect_key_released(glib::clone!(@weak self as widget => @default-panic, move |_, key, _, _| 
            match key {
                gdk::Key::Control_L | gdk::Key::Control_R => {
                    widget.set_left_osd_visible(false);
                    widget.ctrl_down.set(false)
                },
                gdk::Key::Shift_L | gdk::Key::Shift_R => widget.shift_down.set(false),
                _ => ()
            }
        ));
        self.drawing_area.add_controller(&key_controller);
    }

    fn init_scrolling(&self) {
        let scroll_controller = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        scroll_controller.connect_scroll(glib::clone!(@weak self as widget => @default-panic, move |_, _, y| {
            widget.renderer.borrow_mut().zoom(if y > 0. { 0.9 } else { 1.1 }, Some(widget.mouse_position.get()));
            widget.drawing_area.queue_draw();

            gtk::Inhibit(true)
        }));
        self.drawing_area.add_controller(&scroll_controller);
    }

    fn initialize(&self) {
        self.drawing_area.set_draw_func(glib::clone!(@weak self as widget => move |area, context, width, height|
            widget.plot_provider.borrow().with_mut(|plot| 
                widget.renderer.borrow_mut()
                    .callback(plot, *widget.editor_mode.borrow(), area, context, width, height)
                    .map(|_| ())
                    .unwrap_or_die()
            );
        ));

        self.drawing_area.set_focusable(true);
        self.drawing_area.grab_focus();
        self.drawing_area.set_focus_on_click(true);

        self.init_buttons();
        self.init_mouse();
        self.init_keyboard();
        self.init_scrolling();
        self.init_context_menu();
    }

    fn on_mouse_move(&self, x: f64, y: f64) {
        let position = Vector2(x, y);
        self.mouse_position.set(position);

        self.plot_provider.borrow_mut().with_mut(|plot|
            if let Selection::MoveBlock(block) = plot.selection_mut() {

                let mut position = VectorCast::cast(self.renderer.borrow().screen_to_world(position));
                
                let editor_mode = self.editor_mode.borrow();
                if matches!(*editor_mode, EditorMode::Grid) {
                    position = position / GRID_SIZE.into() * GRID_SIZE.into();    
                }
                
                block.set_position(position);
                self.drawing_area.queue_draw();
            }
        );
    }

    fn context_menu(&self, x: f64, y: f64) {        
        let position = self.renderer.borrow().screen_to_world(Vector2(x, y));

        self.plot_provider.borrow_mut().with_mut(|plot| {
            match plot.get_block_at(VectorCast::cast(position)) {
                Some(id) => {
                    if let Some(block) = plot.get_block_mut(id) {
                        block.set_highlighted(true);
                        let start_position = block.position();

                        if let Selection::None = plot.selection() {
                            plot.unhighlight();
                            plot.set_selection(Selection::Single(Selectable::Block(id), start_position));
                        }

                        //drop(plot);

                        self.context_menu.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
                        self.context_menu.popup();
                    }
                }
                None => {
                    self.area_context_menu.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
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

    fn selection_shift_click(&self, selected: Vec<Selectable>, position: Vector2<i32>) -> bool {
        self.plot_provider.borrow().with_mut(move |p| 
            if let Some(block_id) = p.get_block_at(position) {
                let mut selected = selected.clone();
                if let Some(index) = selected.iter().position(|sel| sel == &Selectable::Block(block_id)) {
                    selected.remove(index);
                    p.get_block_mut(block_id).unwrap().set_highlighted(false);
                }
                else {
                    p.get_block_mut(block_id).unwrap().set_highlighted(true);
                    selected.push(Selectable::Block(block_id));
                }
                p.set_selection(Selection::Many(selected));
                
                true
            }
            else if let Some(waypoint_id) = p.get_waypoint_at(position) {
                let mut selected = selected.clone();
                if let Some(index) = selected.iter().position(|sel| sel == &Selectable::Waypoint(waypoint_id.clone())) {
                    selected.remove(index);
                    p.get_connection_mut(waypoint_id.connection_id()).unwrap().get_segment_mut(waypoint_id.location()).unwrap().set_highlighted(false);
                }   
                else {
                    p.get_connection_mut(waypoint_id.connection_id()).unwrap().get_segment_mut(waypoint_id.location()).unwrap().set_highlighted(true);
                    selected.push(Selectable::Waypoint(waypoint_id));
                }
                p.set_selection(Selection::Many(selected));

                true
            }
            else {
                false
            }
        ).unwrap_or(false)
    }

    fn drag_begin(&self, position: Vector2<i32>) {
        let selection = self.plot_provider.borrow().with(|p| p.selection().clone());
        match selection {
            Some(Selection::MoveBlock(block)) =>
                self.application.borrow()
                    .new_action(Action::NewBlock(self.plot_provider.borrow().clone(), block.clone())),
            Some(Selection::Many(block_ids)) => 
                if self.shift_down.get() {
                    if self.selection_shift_click(block_ids, position) {
                        self.drawing_area.queue_draw();
                        return;
                    }
                }
            Some(Selection::Single(block_id, _)) =>
                if self.shift_down.get() {
                    if self.selection_shift_click(vec![block_id], position) {
                        self.drawing_area.queue_draw();
                        return;
                    }
                }
            _ => ()   
        }

        self.plot_provider.borrow().with_mut(|plot| {
            plot.unhighlight();
            if let Some(id) = plot.get_block_at(position) {
                let block = plot.get_block_mut(id).unwrap();

                if block.on_mouse_press(position) {
                    plot.set_selection(Selection::MouseEvent(id));
                    plot.add_block_to_update(id);
                }
                else if let Some(i) = block.position_on_connection(position, false) {
                    let start = block.get_connector_pos(Connector::Output(i));
                    plot.set_selection(Selection::Connection(ConnectionSource::Block(id, i), start, start));
                }
                else {
                    let start_position = block.position();
                    block.set_highlighted(true);
                    plot.set_selection(Selection::Single(Selectable::Block(id), start_position));
                }
            }
            else if let Some(id) = plot.get_waypoint_at(position) {
                let waypoint = plot.get_connection_mut(id.connection_id()).and_then(|c| c.get_segment_mut(id.location())).unwrap();
                let start = *waypoint.position().unwrap();

                if self.shift_down.take() {
                    waypoint.set_highlighted(true);
                    plot.set_selection(Selection::Single(Selectable::Waypoint(id), start))
                }
                else {
                    plot.set_selection(Selection::Connection(ConnectionSource::Waypoint(id), start, start))                    
                }
            }
            else {
                plot.set_selection(Selection::Area(position, position));
            }
        });
    
        self.drawing_area.queue_draw();
    }
        
    fn drag_update(&self, offset: Vector2<i32>) {
        self.plot_provider.borrow().with_mut(|plot|
            match plot.selection().clone() {
                Selection::Single(selected, Vector2(start_x, start_y)) => {
                    let editor_mode = self.editor_mode.borrow();
                    let new_position = if matches!(*editor_mode, EditorMode::Grid) {
                        (Vector2(start_x, start_y) + offset) / GRID_SIZE.into() * GRID_SIZE.into()
                    } else {
                        Vector2(start_x, start_y) + offset
                    };

                    match selected {
                        Selectable::Block(id) => {
                            let block = plot.get_block_mut(id);
                            if block.is_none() {
                                plot.set_selection(Selection::None);
                                return;
                            }
                        
                            block.unwrap().set_position(new_position);
                        }
                        Selectable::Waypoint(id) => {
                            let waypoint = plot.get_connection_mut(id.connection_id()).and_then(|c| c.get_segment_mut(id.location()));
                            if waypoint.is_none() {
                                plot.set_selection(Selection::None);
                                return;
                            }

                            waypoint.unwrap().set_position(new_position);
                        }
                    }
                    self.drawing_area.queue_draw();
                }
                Selection::Connection(source, start, _) => {
                    plot.set_selection(Selection::Connection(source, start, start + offset));
                    self.drawing_area.queue_draw();
                }
                Selection::Area(area_start, _) => {
                    plot.set_selection(Selection::Area(area_start, area_start + offset));
                    self.drawing_area.queue_draw();
                }
                _ => ()
            }
        );
    }

    fn drag_end(&self, offset: Vector2<i32>) {
        let plot_provider = self.plot_provider.borrow();
        let selection = plot_provider.with(|plot| plot.selection().clone()).unwrap();
        match selection {
            Selection::Single(selected, Vector2(start_x, start_y)) => {
                if offset.0 == 0 && offset.1 == 0 {
                    return;
                }

                let editor_mode = self.editor_mode.borrow();
                let new_position = if matches!(*editor_mode, EditorMode::Grid) {
                    (Vector2(start_x, start_y) + offset) / GRID_SIZE.into() * GRID_SIZE.into()
                } else {
                    Vector2(start_x, start_y) + offset
                };

                match selected {
                    Selectable::Block(block_id) => self.application.borrow().new_action(Action::MoveBlock(plot_provider.clone(), block_id, Vector2(start_x, start_y), new_position)),
                    Selectable::Waypoint(id) => {
                        let con_action = plot_provider.with_mut(|plot| {
                            let block_id = plot.get_block_at(new_position)?;
                            let port = plot.get_block(block_id)?.position_on_connection(new_position, true)?;

                            let segment = plot.get_connection(id.connection_id())?.get_segment(id.location())?;
                            Some(Action::WaypointToConnection(plot_provider.clone(), id.clone(), segment.clone(), block_id, port))
                        }).flatten();
                        
                        self.application.borrow().new_action(if let Some(con_action) = con_action {
                            con_action
                        }
                        else {
                            Action::MoveWaypoint(plot_provider.clone(), id, Vector2(start_x, start_y), new_position)
                        })
                    }
                }
            },
            Selection::Connection(ConnectionSource::Block(block_id, output), _, position) => {
                let connection = plot_provider.with_mut(|plot| {
                    plot.set_selection(Selection::None);
                    plot.get_block_at(position)
                        .and_then(|id| plot.get_block(id)
                            .unwrap().position_on_connection(position, true)
                            .map(|i| (id, i)))
                        .map_or_else(
                            || Connection::new(Port::Output(block_id, output), vec![Segment::Waypoint(vec![], position, false)]),
                            |(block, i)| Connection::new_basic(block_id, output, block, i )
                        )
                });
                if let Some(connection) = connection {
                    self.application.borrow().new_action(Action::NewConnection(plot_provider.clone(), connection));
                }

                self.drawing_area.queue_draw()
            }
            Selection::Connection(ConnectionSource::Waypoint(segment_id), _, position) => {
                let segment = plot_provider.with_mut(|plot| {
                    plot.set_selection(Selection::None);
                    if let Some((block_id, i)) = plot.get_block_at(position).and_then(|id| plot.get_block(id).unwrap().position_on_connection(position, true).map(|i| (id, i))) {
                        Segment::Block(block_id, i)
                    }   
                    else {
                        Segment::Waypoint(vec![], position, false)
                    }                
                });
                if let Some(segment) = segment {
                    self.application.borrow().new_action(Action::AddSegment(plot_provider.clone(), segment_id, segment, None))
                }

                self.drawing_area.queue_draw();
            }
            Selection::Area(_, _) => {
                plot_provider.with_mut(|plot| plot.highlight_area());
                self.drawing_area.queue_draw()
            }
            Selection::MouseEvent(block_id) => {
                plot_provider.with_mut(|plot| {
                    plot.set_selection(Selection::None);
                    plot.get_block_mut(block_id)
                        .map(|block| block.on_mouse_release());
                    plot.add_block_to_update(block_id);
                });
                self.drawing_area.queue_draw();
            }
            _ => {}
        };
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
        self.left_osd_label.set_label("0, 0");
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
