use gl::backend::Backend;
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
    gdk::GLContext,
    gio::{ActionGroup, ActionMap},
    prelude::{GLAreaExt, InitializingWidgetExt, WidgetExt},
    subclass::{
        prelude::{GLAreaImpl, WidgetImpl},
        widget::{CompositeTemplate, WidgetImplExt},
    },
    Accessible, Buildable, CompositeTemplate, ConstraintTarget, GLArea, Native, Root,
    ShortcutManager, Widget,
};

use std::cell::RefCell;

use crate::{renderer::Renderer, application::Application};

wrapper! {
    pub struct CircuitView(ObjectSubclass<CircuitViewTemplate>)
        @extends GLArea, Widget,
        @implements ActionGroup, ActionMap, Accessible, Buildable, ConstraintTarget, Native, Root, ShortcutManager;
}

impl CircuitView {
    pub fn new(app: &Application) -> Self {
        glib::Object::new(&[("application", app)]).unwrap()
    }
}

unsafe impl Backend for CircuitView {
    fn swap_buffers(&self) -> Result<(), gl::SwapBuffersError> {
        // We're supposed to draw (and hence swap buffers) only inside the `render()` vfunc or
        // signal, which means that GLArea will handle buffer swaps for us.
        Ok(())
    }

    unsafe fn get_proc_address(&self, symbol: &str) -> *const std::ffi::c_void {
        epoxy::get_proc_addr(symbol)
    }

    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        let scale = self.scale_factor();
        let width = self.width();
        let height = self.height();
        ((width * scale) as u32, (height * scale) as u32)
    }

    fn is_current(&self) -> bool {
        match self.context() {
            Some(context) => GLContext::current() == Some(context),
            None => false,
        }
    }

    unsafe fn make_current(&self) {
        GLAreaExt::make_current(self);
    }
}

#[derive(CompositeTemplate, Default)]
#[template(resource = "/app/circuit-view.ui")]
pub struct CircuitViewTemplate {
    renderer: RefCell<Option<Renderer>>,
}

#[object_subclass]
impl ObjectSubclass for CircuitViewTemplate {
    const NAME: &'static str = "CircuitView";
    type Type = CircuitView;
    type ParentType = GLArea;

    fn class_init(my_class: &mut Self::Class) {
        Self::bind_template(my_class);
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for CircuitViewTemplate {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj)
    }
}

impl WidgetImpl for CircuitViewTemplate {
    fn realize(&self, widget: &Self::Type) {
        self.parent_realize(widget);

        let context =
            unsafe { gl::backend::Context::new(widget.clone(), true, Default::default()) }.unwrap();
        *self.renderer.borrow_mut() = Some(Renderer::new(context))
    }

    fn unrealize(&self, widget: &Self::Type) {
        *self.renderer.borrow_mut() = None;

        self.parent_unrealize(widget);
    }
}

impl GLAreaImpl for CircuitViewTemplate {
    fn render(&self, _gl_area: &Self::Type, _context: &GLContext) -> bool {
        self.renderer.borrow().as_ref().unwrap().draw();

        true
    }
}
