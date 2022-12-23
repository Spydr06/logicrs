use crate::{renderer::{
    Renderable,
    Renderer
}, application::data::ApplicationData};
use super::Connector;
use std::sync::{atomic::{AtomicBool, Ordering}};
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Linkage {
    pub block_id: u32,
    pub port: u8,
}

#[derive(Debug, Default, Serialize, Deserialize)]
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
    fn render<R>(&self, renderer: &R, data: &ApplicationData) -> Result<(), R::Error>
        where R: Renderer
    {
        let plot = data.current_plot();
        let from = plot.get_block(self.from.block_id);
        let to = plot.get_block(self.to.block_id);
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
        renderer.move_to(start)
            .curve_to(offset.0, offset.1, end)
            .stroke()
    }
}
