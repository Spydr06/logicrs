use crate::{simulator::*, selection::*};
use serde::{Serialize, Deserialize};

use super::action::Action;

#[derive(Serialize, Deserialize, Debug)]
pub enum Clipboard {
    Empty,
    Blocks(Vec<Block>),
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
        if let Clipboard::Blocks(blocks) = self {
            return plot_provider.with_mut(|plot| { 
                let mut blocks = blocks.to_owned();
                blocks.prepare_pasting(plot);
                Action::PasteBlocks(plot_provider.to_owned(), blocks)
            }).ok_or(String::from("no current plot"));
        }
        
        panic!("called `paste_to()` on clipboard != Clipboard::Blocks")
    }
}

impl From<&Plot> for Clipboard {
    fn from(plot: &Plot) -> Self {
        match plot.selection() {
            Selection::Single(block_id) => {
                if let Some(block) = plot.get_block(*block_id) && !block.unique() {
                    let mut block = block.clone();
                    block.prepare_copying(());
                    Self::Blocks(vec![block])
                }
                else {
                    Self::Empty
                }
            },
            Selection::Many(blocks) => {
                let mut blocks = blocks
                    .iter()
                    .filter_map(|block_id| plot.get_block(*block_id).filter(|block| !block.unique()))
                    .map(|block| block.clone())
                    .collect::<Vec<Block>>();
                blocks.prepare_copying(());
                Self::Blocks(blocks)
            },
            _ => Self::Empty
        }
    }
}

trait Copyable<T> {
    fn prepare_copying(&mut self, data: T) -> &mut Self;
}

impl Copyable<()> for Vec<Block> {
    fn prepare_copying(&mut self, _data: ()) -> &mut Self {        
        let ids = self.iter()
            .map(|block| block.id())
            .collect::<Vec<u32>>();
        self.iter_mut().for_each(|block| {
            block.connections_mut()
                .iter_mut()
                .filter(|c| c.is_some() && !ids.contains(&c.as_ref().unwrap().destination_id()))
                .for_each(|c| *c = None);
        });
        self
    }
}

impl Copyable<()> for Block {
    fn prepare_copying(&mut self, _data: ()) -> &mut Self {
        self.connections_mut().iter_mut().for_each(|c| *c = None);
        self
    }
}

trait Pasteable<T> {
    fn prepare_pasting(&mut self, data: T) -> &mut Self;
}

impl Pasteable<&mut Plot> for Vec<Block> {
    fn prepare_pasting(&mut self, plot: &mut Plot) -> &mut Self {
        if self.len() > 0 && self.iter().map(|block| block.id()).min().unwrap() <= plot.current_id() {
            // change ids of the blocks

            for i in 0..self.len() {
                let block = self.get_mut(i).unwrap();
                let old_id = block.id();
                let new_id = plot.next_id();
                block.refactor_id(new_id);
                drop(block);
                
                self.iter_mut().for_each(|block| block
                    .connections_mut()
                    .iter_mut()
                    .for_each(|c| {
                        if let Some(connection) = c && connection.destination_id() == old_id {
                            connection.set_destination_id(new_id);
                        }
                    })
                );
            }
        }

        self
    }
}
