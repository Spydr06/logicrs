use std::{f64, cmp, collections::{HashSet, HashMap}};

use crate::{renderer::{*, vector::Vector2}, selection::SelectionField};
use serde::{Serialize, Deserialize};

use super::*;

pub enum Connector {
    Input(u8),
    Output(u8)
}

pub type BlockID = uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    id: BlockID,
    name: String,

    position: Vector2<i32>,
    size: Vector2<i32>,

    #[serde(skip)]
    highlighted: bool,
    unique: bool,
    passthrough: bool,

    inputs: Vec<Option<ConnectionID>>,
    outputs: Vec<Option<ConnectionID>>,

    state: State,
    
    decoration: Decoration,
}

impl Identifiable for Block {
    type ID = BlockID;
}

impl Block {
    pub const MAX_CONNECTIONS: u8 = 128;

    pub fn new_sized(module: &&Module, position: Vector2<i32>, unique: bool, num_inputs: u8, num_outputs: u8) -> Self {
        let name = module.name().clone();
        Self {
            id: crate::new_uuid(),
            position,
            size: Vector2(
                cmp::max(75, (name.len() * 10) as i32),
                cmp::max(num_inputs, num_outputs) as i32 * 25 + 50
            ),
            highlighted: false,
            unique,
            passthrough: true,
            inputs: vec![None; num_inputs as usize],
            outputs: vec![None; num_outputs as usize],
            name,
            state: if module.builtin() { State::Direct(0) } else { State::Inherit(PlotState::default()) },
            decoration: module.decoration().clone(),
        }
    }

    pub fn new(module: &&Module, position: Vector2<i32>) -> Self {
        Self::new_sized(module, position, false, module.get_num_inputs(), module.get_num_outputs())
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn unique(&self) -> bool {
        self.unique
    }

    pub fn id(&self) -> BlockID {
        self.id
    }

    pub fn set_id(&mut self, id: BlockID) {
        self.id = id;
    }

    pub fn module_id(&self) -> &String {
        &self.name
    }

    pub fn is_in_area(&self, area: ScreenSpace) -> bool {
        !(
            self.position.0 > area.1.0 as i32 || 
            self.position.1 > area.1.1 as i32 ||
            self.position.0 + self.size.0 < area.0.0 as i32 || 
            self.position.1 + self.size.1 < area.0.1 as i32
        )
    }

    pub fn touches(&self, point: Vector2<i32>) -> bool {
        point.0 > self.position.0 - 3 && point.0 < self.position.0 + self.size.0 + 3 &&
        point.1 > self.position.1 - 3 && point.1 < self.position.1 + self.size.1 + 3
    }

    pub fn set_highlighted(&mut self, highlighted: bool) {
        self.highlighted = highlighted;
    }

    pub fn highlighted(&self) -> bool {
        self.highlighted
    }

    pub fn set_position(&mut self, position: Vector2<i32>) {
        self.position = position;
    }

    pub fn position(&self) -> Vector2<i32> {
        self.position
    }

    pub fn size(&self) -> Vector2<i32> {
        self.size
    }

    pub fn connected_to(&self) -> Vec<ConnectionID> {
        self.inputs.iter().chain(self.outputs.iter()).filter_map(|a| *a).collect()
    }

    pub fn connections_mut(&mut self) -> impl Iterator<Item = &mut Option<ConnectionID>> {
        self.inputs.iter_mut().chain(self.outputs.iter_mut())
    }

    pub fn get_connector_pos(&self, connector: Connector) -> Vector2<i32> {
        match connector {
            Connector::Input(i) => Vector2(self.position.0, self.position.1 + 25 * i as i32 + 50),
            Connector::Output(i) => Vector2(self.position.0 + self.size.0, self.position.1 + 25 * i as i32 + 50)
        }
    }

    pub fn set_connection(&mut self, port: Port, connection: Option<ConnectionID>) -> &mut Self {
        match port {
            Port::Input(index) => self.inputs[index as usize] = connection,
            Port::Output(index) => self.outputs[index as usize] = connection
        }
        self
    }

    pub fn outputs(&self) -> &Vec<Option<ConnectionID>> {
        &self.outputs
    }
    
    pub fn outputs_mut(&mut self) -> &mut Vec<Option<ConnectionID>> {
        &mut self.outputs
    }

    pub fn inputs(&self) -> &Vec<Option<ConnectionID>> {
        &self.inputs
    }

    pub fn inputs_mut(&mut self) -> &mut Vec<Option<ConnectionID>> {
        &mut self.inputs
    }

    pub fn is_active(&self) -> bool {
        self.decoration.is_active()
    }

    pub fn set_active(&mut self, is_active: bool) {
        self.decoration.set_active(is_active)
    }

    #[inline(always)]
    pub fn state(&self) -> &State {
        &self.state
    }

    #[inline(always)]
    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }

    #[inline(always)]
    pub fn set_bytes(&mut self, bytes: u128) {
        self.state = State::Direct(bytes);
    }

    #[inline(always)]
    pub fn bytes(&self) -> u128 {
        match self.state {
            State::Direct(bytes) => bytes,
            _ => panic!()
        }
    }

    pub fn set_passthrough(&mut self, passthrough: bool) {
        self.passthrough = passthrough
    }

    pub fn passthrough(&self) -> bool {
        self.passthrough
    }

    pub fn on_mouse_press(&mut self, mut position: Vector2<i32>) -> bool {
        position -= self.position;
        if position.0 > 15 && position.1 > 25 && 
            position.0 < self.size.0 - 15 && position.1 < self.size.1 - 10 {
            self.decoration.on_mouse_press()
        }
        else {
            false
        }
    }

    pub fn on_mouse_release(&mut self) {
        self.decoration.on_mouse_release();
    }

    pub fn position_on_connection(&self, position: Vector2<i32>, is_input: bool) -> Option<u8> {
        if is_input {
            for i in 0..self.inputs.len() {
                let connector_pos = (self.position.0, self.position.1 + 25 * i as i32 + 50);
                if (position.0 - connector_pos.0).abs() < Connection::HITBOX_SIZE && (position.1 - connector_pos.1).abs() < Connection::HITBOX_SIZE {
                    return Some(i as u8);
                }
            }
        }
        else {
            for i in 0..self.outputs.len() {
                let connector_pos = (self.position.0 + self.size.0, self.position.1 + 25 * i as i32 + 50);
                if (position.0 - connector_pos.0).abs() < Connection::HITBOX_SIZE && (position.1 - connector_pos.1).abs() < Connection::HITBOX_SIZE {
                    return Some(i as u8);
                }
            }
        }
        None
    }

    pub fn simulate(&mut self, connections: &mut HashMap<ConnectionID, Connection>, to_update: &mut HashSet<BlockID>, project: &mut Project, call_stack: &mut HashSet<String>) -> SimResult<()> {
        // collect input states
        let inputs = self.inputs.collect(connections);
    
        let mut_ref_ptr = project as *mut Project;
        if let Some(module) = project.module_mut(&self.name) {
            // simulate the block
            let outputs = module.simulate(inputs, self, unsafe { &mut *mut_ref_ptr }, call_stack)?;

            // dissect output state
            for (i, connection_id) in self.outputs.iter().enumerate() {
                if let Some(connection) = connection_id.map(|connection_id| connections.get_mut(&connection_id)).flatten() {
                    let active = (outputs >> i as u128) & 1 != 0;
                    if active != connection.is_active() {
                        to_update.insert(connection.destination_id());
                        connection.set_active(active);
                    }
                }
            }
        }
        else {
            error!("no module named {} found", self.name);
        }

        Ok(())
    }
}


impl Renderable for Block {
    fn render<R>(&self, renderer: &R, plot: &Plot) -> Result<(), R::Error>
        where R: Renderer 
    {
        renderer.set_line_width(2.);
        renderer.rounded_rect(self.position, self.size, 5)
            .set_color(unsafe { &COLOR_THEME.block_bg_color }).fill()?;

        renderer.top_rounded_rect(self.position, Vector2(self.size.0, 25), 5)
            .set_color(unsafe { &COLOR_THEME.border_color })
            .fill()?;

        renderer.move_to(Vector2(self.position.0 + 5, self.position.1 + 18))
            .set_color(unsafe { &COLOR_THEME.block_fg_color })
            .show_text(self.name.as_str())?;

        renderer.rounded_rect(self.position, self.size, 5);
        match self.highlighted {
            true => renderer.set_color(unsafe { &COLOR_THEME.accent_fg_color }),
            false => renderer.set_color(unsafe { &COLOR_THEME.border_color })    
        };
        renderer.stroke()?;

        let show_suggestion = plot.selection().connecting();
        let connector = |position, is_input|
            renderer
                .arc(position, 6., 0., f64::consts::TAU)
                .set_color(unsafe {if show_suggestion && is_input { &COLOR_THEME.suggestion_fg_color } else { &COLOR_THEME.disabled_fg_color }} )
                .fill_preserve()?
                .set_color(unsafe {if self.highlighted { &COLOR_THEME.accent_fg_color } else { &COLOR_THEME.border_color }}).stroke();

        renderer.set_line_width(1.);
        for (i, _) in self.inputs.iter().enumerate().filter(|(_, c)| c.is_none()) {
            connector(Vector2(self.position.0, self.position.1 + 25 * i as i32 + 50), true)?;
        }

        for (i, _) in self.outputs.iter().enumerate().filter(|(_, c)| c.is_none()) {
            connector(Vector2(self.position.0 + self.size.0, self.position.1 + 25 * i as i32 + 50), false)?;
        }

        self.decoration.render(renderer, self).map(|_| ())
    }
}