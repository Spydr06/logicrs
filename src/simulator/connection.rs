use crate::renderer::*;
use super::{Connector, Plot};
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Linkage {
    pub block_id: u32,
    pub port: u8,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Connection {
    from: Linkage,
    to: Linkage,
    active: bool
}

impl Connection {
    pub fn new(from: Linkage, to: Linkage) -> Self {
        Self {
            from,
            to,
            active: false
        }
    }

    pub fn contains(&self, id: u32) -> bool {
        self.from.block_id == id || self.to.block_id == id
    }
}

impl Renderable for Connection {
    fn render<R>(&self, renderer: &R, plot: &Plot) -> Result<(), R::Error>
        where R: Renderer
    {
        let from = plot.get_block(self.from.block_id);
        let to = plot.get_block(self.to.block_id);
        if from.is_none() || to.is_none() {
            return Ok(())
        }
        if self.active {
            renderer.set_color(&DEFAULT_THEME.enabled_bg_color);
        }
        else {
            renderer.set_color(&DEFAULT_THEME.disabled_bg_color);
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
