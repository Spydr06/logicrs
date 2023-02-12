use crate::renderer::*;
use super::{Connector, Plot, BlockID};
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Linkage {
    pub block_id: BlockID,
    pub port: u8,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Connection {
    from: Linkage,
    to: Linkage,
    active: bool
}

impl Connection {
    pub const HITBOX_SIZE: i32 = 8;

    pub fn new(from: Linkage, to: Linkage) -> Self {
        Self {
            from,
            to,
            active: false
        }
    }

    pub fn contains(&self, id: BlockID) -> bool {
        self.from.block_id == id || self.to.block_id == id
    }

    pub fn origin(&self) -> u8 {
        self.from.port
    }

    pub fn origin_id(&self) -> BlockID {
        self.from.block_id
    }

    pub fn set_origin_id(&mut self, origin_id: BlockID) {
        self.from.block_id = origin_id;
    }

    pub fn destination_id(&self) -> BlockID {
        self.to.block_id
    }

    pub fn set_destination_id(&mut self, block_id: BlockID) {
        self.to.block_id = block_id;
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
            renderer.set_color(unsafe { &COLOR_THEME.enabled_bg_color });
        }
        else {
            renderer.set_color(unsafe { &COLOR_THEME.disabled_bg_color });
        }
        let start = from.unwrap().get_connector_pos(Connector::Output(self.from.port));
        let end = to.unwrap().get_connector_pos(Connector::Input(self.to.port));
        let offset = (
            (start.0 + ((end.0 - start.0) as f32 * 0.7) as i32, start.1),
            (end.0 + ((start.0 - end.0) as f32 * 0.7) as i32, end.1),
        );
        renderer.move_to(start)
            .curve_to(offset.0, offset.1, end)
            .stroke().map(|_| ())
    }
}
