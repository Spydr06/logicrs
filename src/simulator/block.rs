use std::{f64, cmp};

use crate::{renderer::*, selection::SelectionField};
use serde::{Serialize, Deserialize};

use super::{Connection, Linkage, Plot, Decoration, Module};

pub enum Connector {
    Input(u8),
    Output(u8)
}

pub type BlockID = u32;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    id: BlockID,
    name: String,

    position: (i32, i32),
    size: (i32, i32),

    #[serde(skip)]
    start_pos: (i32, i32), // starting position of drag movements

    #[serde(skip)]
    highlighted: bool,

    deleteable: bool,

    num_inputs: u8,
    num_outputs: u8,
    connections: Vec<Option<Connection>>,

    decoration: Decoration,
}

impl Block {
    pub fn new_sized(module: &&Module, position: (i32, i32), id: BlockID, deleteable: bool, num_inputs: u8, num_outputs: u8) -> Self {
        let mut connections = Vec::with_capacity(num_outputs as usize);
        (0..num_outputs).for_each(|_| connections.push(None));

        let name = module.name().clone();

        Self {
            id,
            position,
            start_pos: (0, 0),
            size: (
                cmp::max(75, (name.len() * 10) as i32),
                cmp::max(num_inputs, num_outputs) as i32 * 25 + 50
            ),
            highlighted: false,
            deleteable,
            num_inputs,
            num_outputs,
            name,
            connections,
            decoration: module.decoration().clone(),
        }
    }

    pub fn new(module: &&Module, position: (i32, i32), id: BlockID) -> Self {
        Self::new_sized(module, position, id, true, module.get_num_inputs(), module.get_num_outputs())
    }

    pub fn deleteable(&self) -> bool {
        self.deleteable
    }

    pub fn id(&self) -> BlockID {
        self.id
    }

    pub fn refactor_id(&mut self, id: BlockID) -> &mut Self {
        self.id = id;
        
        self.connections.iter_mut().for_each(|c| 
            if let Some(connection) = c {
                connection.set_origin_id(id);
            }
        );

        self
    }

    pub fn module_id(&self) -> &String {
        &self.name
    }

    pub fn is_in_area(&self, area: (i32, i32, i32, i32)) -> bool {
        !(
            self.position.0 > area.2 || 
            self.position.1 > area.3 ||
            self.position.0 + self.size.0 < area.0 || 
            self.position.1 + self.size.1 < area.1
        )
    }

    pub fn touches(&self, point: (i32, i32)) -> bool {
        point.0 > self.position.0 - 3 && point.0 < self.position.0 + self.size.0 + 3 &&
        point.1 > self.position.1 - 3 && point.1 < self.position.1 + self.size.1 + 3
    }

    pub fn set_highlighted(&mut self, highlighted: bool) {
        self.highlighted = highlighted;
    }

    pub fn highlighted(&self) -> bool {
        self.highlighted
    }

    pub fn set_position(&mut self, position: (i32, i32)) {
        self.position = position;
    }

    pub fn position(&self) -> (i32, i32) {
        self.position
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    pub fn set_start_pos(&mut self, start_pos: (i32, i32)) {
        self.start_pos = start_pos
    }

    pub fn start_pos(&self) -> (i32, i32) {
        self.start_pos
    }

    pub fn get_connector_pos(&self, connector: Connector) -> (i32, i32) {
        match connector {
            Connector::Input(i) => (self.position.0, self.position.1 + 25 * i as i32 + 50),
            Connector::Output(i) => (self.position.0 + self.size.0, self.position.1 + 25 * i as i32 + 50)
        }
    }

    pub fn add_connection(&mut self, port: u8, connection: Connection) -> &mut Self {
        self.connections[port as usize] = Some(connection);
        self
    }

    pub fn remove_connection(&mut self, port: u8) -> &mut Self {
        self.connections[port as usize] = None;
        self
    }

    pub fn connect_to(&mut self, port: u8, to: Linkage) -> &mut Self {
        self.connections[port as usize] = Some(Connection::new(
            Linkage {block_id: self.id, port},
            to
        ));
        self
    }

    pub fn connections(&self) -> &Vec<Option<Connection>> {
        &self.connections
    }

    pub fn connections_mut(&mut self) -> &mut Vec<Option<Connection>> {
        &mut self.connections
    }

    pub fn position_on_connection(&self, position: (i32, i32), is_input: bool) -> Option<u8> {
        if is_input {
            for i in 0..self.num_inputs {
                let connector_pos = (self.position.0, self.position.1 + 25 * i as i32 + 50);
                if (position.0 - connector_pos.0).abs() < Connection::HITBOX_SIZE && (position.1 - connector_pos.1).abs() < Connection::HITBOX_SIZE {
                    return Some(i);
                }
            }
        }
        else {
            for i in 0..self.num_outputs {
                let connector_pos = (self.position.0 + self.size.0, self.position.1 + 25 * i as i32 + 50);
                if (position.0 - connector_pos.0).abs() < Connection::HITBOX_SIZE && (position.1 - connector_pos.1).abs() < Connection::HITBOX_SIZE {
                    return Some(i);
                }
            }
        }
        None
    }

    fn draw_connector<R>(&self, renderer: &R, position: (i32, i32), highlighted: bool) -> Result<(), R::Error>
        where R: Renderer
    {
        renderer
            .arc(position, 6., 0., f64::consts::TAU)
            .set_color(if highlighted { &DEFAULT_THEME.suggestion_fg_color } else { &DEFAULT_THEME.disabled_fg_color })
            .fill_preserve()?
            .set_color(if self.highlighted { &DEFAULT_THEME.accent_fg_color } else { &DEFAULT_THEME.border_color })
            .stroke()
            .map(|_| ())
    }
}


impl Renderable for Block {
    fn render<R>(&self, renderer: &R, plot: &Plot) -> Result<(), R::Error>
        where R: Renderer 
    {
        renderer.set_line_width(2.);
        renderer.rounded_rect(self.position, self.size, 5)
            .set_color(&DEFAULT_THEME.block_bg_color).fill()?;

        renderer.top_rounded_rect(self.position, (self.size.0, 25), 5)
            .set_color(&DEFAULT_THEME.border_color)
            .fill()?;

        renderer.move_to((self.position.0 + 5, self.position.1 + 18))
            .set_color(&DEFAULT_THEME.block_fg_color)
            .show_text(self.name.as_str())?;

        renderer.rounded_rect(self.position, self.size, 5);
        match self.highlighted {
            true => renderer.set_color(&DEFAULT_THEME.accent_fg_color),
            false => renderer.set_color(&DEFAULT_THEME.border_color)    
        };
        renderer.stroke()?;

        renderer.set_line_width(1.);
        let highlight_inputs = plot.selection().connecting();
        for i in 0..self.num_inputs {
            self.draw_connector(renderer, (self.position.0, self.position.1 + 25 * i as i32 + 50), highlight_inputs)?;
        }

        for i in 0..self.num_outputs {
            self.draw_connector(renderer, (self.position.0 + self.size.0, self.position.1 + 25 * i as i32 + 50), false)?;
        }

        self.decoration.render(renderer, self).map(|_| ())
    }
}