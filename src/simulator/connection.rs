use crate::renderer::*;
use super::*;
use serde::{Serialize, Deserialize};

pub type ConnectionID = uuid::Uuid;

#[derive(Copy, Clone)]
pub enum Port {
    Input(u8),
    Output(u8)
}

impl Port {
    pub fn index(&self) -> u8 {
        match self {
            Self::Input(index) => *index,
            Self::Output(index) => *index,
        }
    } 
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub struct Linkage {
    pub block_id: BlockID,
    pub port: u8,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Connection {
    id: ConnectionID,
    from: Linkage,
    to: Linkage,
    active: bool
}

impl Identifiable for Connection {
    type ID = ConnectionID;
}

impl Connection {
    pub const HITBOX_SIZE: i32 = 8;

    pub fn new(from: Linkage, to: Linkage) -> Self {
        Self {
            id: crate::new_uuid(),
            from,
            to,
            active: false
        }
    }

    pub fn id(&self) -> ConnectionID {
        self.id
    }

    pub fn to(&self) -> Linkage {
        self.from
    }

    pub fn from(&self) -> Linkage {
        self.to
    }

    pub fn to_port(&self) -> Port {
        Port::Input(self.to.port)
    }

    pub fn from_port(&self) -> Port {
        Port::Output(self.from.port)
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
