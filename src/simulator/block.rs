use std::{
    sync::atomic::{AtomicU32, Ordering}, 
    f64,
    cmp
};
use gtk::cairo::Error;

use crate::{
    modules::Module,
    renderer::{
        Renderer,
        Renderable
    }
};
use serde::{Serialize, Deserialize};

use super::{Connection, Linkage};

pub enum Connector {
    Input(u8),
    Output(u8)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    id: u32,
    position: (i32, i32),
    start_pos: (i32, i32), // starting position of drag movements
    size: (i32, i32),
    highlighted: bool,
    num_inputs: u8,
    num_outputs: u8,
    name: String,
    connections: Vec<Option<Connection>>
}

impl Block {
    pub fn new(module: &&Module, position: (i32, i32)) -> Self {
        static ID: AtomicU32 = AtomicU32::new(0u32);

        let num_inputs = module.get_num_inputs();
        let num_outputs = module.get_num_outputs();
        let mut connections = Vec::with_capacity(num_outputs as usize);
        for _ in 0..num_outputs {
            connections.push(None);
        }

        Self {
            id: ID.fetch_add(1u32, Ordering::SeqCst),
            position,
            start_pos: (0, 0),
            size: (75, cmp::max(num_inputs, num_outputs) as i32 * 25 + 50),
            highlighted: false,
            num_inputs,
            num_outputs,
            name: module.name().clone(),
            connections,
        }
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
        point.0 > self.position.0 && point.0 < self.position.0 + self.size.0 &&
        point.1 > self.position.1 && point.1 < self.position.1 + self.size.1
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

    fn draw_connector(&self, renderer: &impl Renderer, position: (i32, i32)) -> Result<(), Error> {
        renderer.arc(position, 6., 0., f64::consts::TAU);
        match self.highlighted {
            true => renderer.set_color(0.2078, 0.5176, 0.894, 1.),
            false => renderer.set_color(0.23, 0.23, 0.23, 1.)       
        };
        
        renderer.fill()?;
    
        renderer.arc(position, 5., 0., f64::consts::TAU);
        renderer.set_color(0.5, 0.1, 0.7, 1.);
        renderer.fill()?;
        
        Ok(())
    }
}


impl Renderable for Block {
    fn render(&self, renderer: &impl Renderer) -> Result<(), Error> {
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

        for i in 0..self.num_inputs {
            self.draw_connector(renderer, (self.position.0, self.position.1 + 25 * i as i32 + 50))?;
        }

        for i in 0..self.num_outputs {
            self.draw_connector(renderer, (self.position.0 + self.size.0, self.position.1 + 25 * i as i32 + 50))?;
        }

        Ok(())
    }
}