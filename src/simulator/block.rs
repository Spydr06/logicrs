use std::{f64, cmp};

use crate::{modules::Module, renderer::*};
use serde::{Serialize, Deserialize};

use super::{Connection, Linkage, Plot};

pub enum Connector {
    Input(u8),
    Output(u8)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    id: u32,
    name: String,

    position: (i32, i32),
    start_pos: (i32, i32), // starting position of drag movements
    size: (i32, i32),

    #[serde(skip)]
    highlighted: bool,
    
    num_inputs: u8,
    num_outputs: u8,
    connections: Vec<Option<Connection>>
}

impl Block {
    pub fn new_sized(module: &&Module, position: (i32, i32), id: u32, num_inputs: u8, num_outputs: u8) -> Self {
        let mut connections = Vec::with_capacity(num_outputs as usize);
        for _ in 0..num_outputs {
            connections.push(None);
        }

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
            num_inputs,
            num_outputs,
            name,
            connections,
        }
    }

    pub fn new(module: &&Module, position: (i32, i32), id: u32) -> Self {
        Self::new_sized(module, position, id, module.get_num_inputs(), module.get_num_outputs())
    }

    pub fn id(&self) -> u32 {
        self.id
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

    pub fn remove_connection(&mut self, index: usize) {
        self.connections.remove(index);
    }

    pub fn position_on_connection(&self, position: (i32, i32), is_input: bool) -> Option<u8> {
        if is_input {
            for i in 0..self.num_inputs {
                let connector_pos = (self.position.0, self.position.1 + 25 * i as i32 + 50);
                if (position.0 - connector_pos.0).abs() < 6 && (position.1 - connector_pos.1).abs() < 6 {
                    return Some(i);
                }
            }
        }
        else {
            for i in 0..self.num_outputs {
                let connector_pos = (self.position.0 + self.size.0, self.position.1 + 25 * i as i32 + 50);
                if (position.0 - connector_pos.0).abs() < 6 && (position.1 - connector_pos.1).abs() < 6 {
                    return Some(i);
                }
            }
        }
        None
    }

    fn draw_connector<R>(&self, renderer: &R, position: (i32, i32)) -> Result<(), R::Error>
        where R: Renderer
    {
        renderer.arc(position, 6., 0., f64::consts::TAU);
        
        renderer.set_color(0.5, 0.1, 0.7, 1.);
        renderer.fill_preserve()?;

        match self.highlighted {
            true => renderer.set_color(0.2078, 0.5176, 0.894, 1.),
            false => renderer.set_color(0.23, 0.23, 0.23, 1.)       
        };   
        renderer.stroke()?;
            
        Ok(())
    }
}


impl Renderable for Block {
    fn render<R>(&self, renderer: &R, _plot: &Plot) -> Result<(), R::Error>
        where R: Renderer 
    {
        renderer.set_line_width(2.);
        renderer.rounded_rect(self.position, self.size, 5);
        
        renderer.set_color(0.13, 0.13, 0.13, 1.).fill()?;
        renderer.top_rounded_rect(self.position, (self.size.0, 25), 5)
            .set_color(0.23, 0.23, 0.23, 1.)
            .fill()?;

        renderer.move_to((self.position.0 + 5, self.position.1 + 18))
            .set_color(1., 1., 1., 1.)
            .show_text(self.name.as_str())?;

        renderer.rounded_rect(self.position, self.size, 5);
        match self.highlighted {
            true => renderer.set_color(0.2078, 0.5176, 0.894, 1.),
            false => renderer.set_color(0.23, 0.23, 0.23, 1.)    
        };
        renderer.stroke()?;

        renderer.set_line_width(1.);
        for i in 0..self.num_inputs {
            self.draw_connector(renderer, (self.position.0, self.position.1 + 25 * i as i32 + 50))?;
        }

        for i in 0..self.num_outputs {
            self.draw_connector(renderer, (self.position.0 + self.size.0, self.position.1 + 25 * i as i32 + 50))?;
        }

        Ok(())
    }
}