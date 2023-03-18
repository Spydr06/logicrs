use crate::{simulator::*, renderer::vector::*};
use serde::{Serialize, Deserialize};

use super::{action::Action, selection::*};

#[derive(Serialize, Deserialize, Debug)]
pub enum Clipboard {
    Empty,
    Blocks(Vec<Block>, Vec<Connection>),
    Module(Module)
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::Empty
    }
}

impl Clipboard {
    pub fn serialize(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|err| err.to_string())
    }

    pub fn deserialize<'a>(data: &'a str) -> Result<Self, String> {
        serde_json::from_str(data).map_err(|err| err.to_string())
    }

    pub fn paste_to(&self, plot_provider: PlotProvider, position: Vector2<f64>) -> Result<Action, String> {
        if let Clipboard::Blocks(blocks, connections) = self {
            let mut data = (blocks.to_owned(), connections.to_owned());
            data.prepare_pasting(position);
            plot_provider.with_mut(|plot| {
                plot.unhighlight();
                plot.set_selection(Selection::Many(data.0.iter().map(|block| block.id()).collect()));
            });
            return Ok(Action::PasteBlocks(plot_provider.to_owned(), data.0, data.1));
        }
        
        panic!("called `paste_to()` on clipboard != Clipboard::Blocks")
    }
}

impl From<&Plot> for Clipboard {
    fn from(plot: &Plot) -> Self {
        match plot.selection() {
            Selection::Single(block_id, _) => {
                if let Some(block) = plot.get_block(*block_id) && !block.unique() {
                    let mut block = block.clone();
                    block.prepare_copying(());
                    Self::Blocks(vec![block], Vec::new())
                }
                else {
                    Self::Empty
                }
            },
            Selection::Many(blocks) => {
                let selection = blocks
                    .iter()
                    .filter_map(|block_id| plot.get_block(*block_id).filter(|block| !block.unique()));
                let block_ids = selection.clone().map(|block| block.id()).collect::<Vec<BlockID>>();
                let blocks = selection.map(|block| block.clone()).collect::<Vec<Block>>();
                let mut data = (blocks, Vec::new());
                data.prepare_copying((plot, block_ids));
                Self::Blocks(data.0, data.1)
            },
            _ => Self::Empty
        }
    }
}

trait Copyable<T> {
    fn prepare_copying(&mut self, data: T) -> &mut Self;
}

impl Copyable<(&Plot, Vec<BlockID>)> for (Vec<Block>, Vec<Connection>) {
    fn prepare_copying(&mut self, data: (&Plot, Vec<BlockID>)) -> &mut Self {
        let plot = data.0;
        let block_ids = data.1;
        let blocks = &mut self.0;
        let connections = &mut self.1;

        blocks.iter_mut().for_each(|block| {
            block.outputs_mut().iter_mut().for_each(|c| {
                if let Some(connection) = c.and_then(|id| plot.get_connection(id)) {
                    if block_ids.contains(&connection.destination_id()) {
                        connections.push(connection.clone());
                    }
                    else {
                        *c = None
                    }
                }
            });

            block.inputs_mut().iter_mut().for_each(|c| {
                if let Some(connection) = c.and_then(|id| plot.get_connection(id)) {
                    if !block_ids.contains(&connection.origin_id()) {
                        *c = None
                    }
                }
            });
        });

        self
    }
}

impl Copyable<()> for Block {
    fn prepare_copying(&mut self, _data: ()) -> &mut Self {
        self.outputs_mut().iter_mut().for_each(|c| *c = None);
        self.inputs_mut().iter_mut().for_each(|c| *c = None);
        self
    }
}

trait Pasteable<T> {
    fn prepare_pasting(&mut self, data: T) -> &mut Self;
}

impl Pasteable<Vector2<f64>> for (Vec<Block>, Vec<Connection>) {
    fn prepare_pasting(&mut self, position: Vector2<f64>) -> &mut Self {
        let min = self.0.iter().map(|block| block.position()).min().unwrap_or_default();
        let offset = Vector2::cast(position) - min;

        self.0.iter_mut().for_each(|block| {
            let old_id = block.id();
            let new_id = crate::new_uuid();
            block.set_id(new_id);
            block.set_position(block.position() + offset);
            block.set_highlighted(true);

            self.1.iter_mut().for_each(|connection| {
                if connection.origin_id() == old_id {
                    connection.set_origin_id(new_id);
                }
                if connection.destination_id() == old_id {
                    connection.set_destination_id(new_id);
                }
            });
        });

        self.1.iter_mut().for_each(|connection| {
            let old_id = connection.id();
            let new_id = crate::new_uuid();
            connection.set_id(new_id);

            self.0.iter_mut().for_each(|block|
                block.connections_mut()
                    .filter(|c| c.is_some())
                    .map(|c| c.as_mut().unwrap())
                    .for_each(|c|
                        if *c == old_id {
                            *c = new_id;
                        }
                )
            );
        });

        self
    }
}
