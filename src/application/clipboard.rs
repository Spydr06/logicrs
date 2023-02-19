use crate::{simulator::*, selection::*};
use serde::{Serialize, Deserialize};

use super::action::Action;

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

    pub fn paste_to(&self, plot_provider: PlotProvider) -> Result<Action, String> {
        if let Clipboard::Blocks(blocks, connections) = self {
            return plot_provider.with_mut(|plot| { 
                let mut data = (blocks.to_owned(), connections.to_owned());
                data.prepare_pasting(plot);
                Action::PasteBlocks(plot_provider.to_owned(), data.0, data.1)
            }).ok_or(String::from("no current plot"));
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
    fn prepare_pasting(&mut self, _data: T) -> &mut Self;
}

impl Pasteable<&mut Plot> for (Vec<Block>, Vec<Connection>) {
    fn prepare_pasting(&mut self, _plot: &mut Plot) -> &mut Self {
        self
    }
}
