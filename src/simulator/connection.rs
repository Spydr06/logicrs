use crate::renderer::{
    Renderable,
    Renderer
};
use super::Connector;
use std::sync::atomic::{
    AtomicBool,
    Ordering
};
use gtk::cairo::Error;

#[derive(Debug, Default)]
pub struct Linkage {
    pub block_id: u32,
    pub port: u8,
}

#[derive(Debug, Default)]
pub struct Connection {
    from: Linkage,
    to: Linkage,
    active: AtomicBool
}

impl Connection {
    pub fn new(from: Linkage, to: Linkage) -> Self {
        Self {
            from,
            to,
            active: AtomicBool::new(false)
        }
    }
}

impl Renderable for Connection {
    fn render(&self, renderer: &impl Renderer) -> Result<(), Error> {
        crate::APPLICATION_DATA.with(|d| {
            let data = d.borrow();

            let from = data.get_block(self.from.block_id);
            let to = data.get_block(self.to.block_id);
            if from.is_none() || to.is_none() {
                return Ok(())
            }

            if self.active.load(Ordering::Relaxed) {
                renderer.set_color(0.0784313, 0.3215686, 0.18745098, 1.);
            }
            else {
                renderer.set_color(0.346, 0.155, 0.41, 1.);
            }

            let start = from.unwrap().get_connector_pos(Connector::Output(self.from.port));
            let end = to.unwrap().get_connector_pos(Connector::Input(self.to.port));
            let offset = (
                (start.0 + ((end.0 - start.0) as f32 * 0.7) as i32, start.1),
                (end.0 + ((start.0 - end.0) as f32 * 0.7) as i32, end.1),
            );
            renderer.move_to(start);
            //renderer.curve_to((end.0, start.1), (start.0, end.1), end);
            renderer.curve_to(offset.0, offset.1, end);
            
            renderer.stroke()
        })
    }
}