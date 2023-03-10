use super::*;

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PlotState {
    blocks: HashMap<BlockID, State>,
    connections: HashMap<ConnectionID, bool>,
}

impl From<&mut Plot> for PlotState {
    fn from(plot: &mut Plot) -> Self {
        Self::from(plot as &Plot)
    }
}

impl From<&Plot> for PlotState {
    fn from(plot: &Plot) -> Self {
        Self {
            blocks: plot.blocks().iter().map(|(id, block)| (*id, block.state().clone())).collect(),
            connections: plot.connections().iter().map(|(id, connection)| (*id, connection.is_active())).collect()
        }
    }
}

impl PlotState {
    pub fn apply(&self, plot: &mut Plot) {
        plot.blocks_mut().iter_mut().for_each(|(id, block)| if let Some(state) = self.blocks.get(id) {
            block.set_state(state.clone())
        });

        plot.connections_mut().iter_mut().for_each(|(id, connection)| if let Some(state) = self.connections.get(id) {
            connection.set_active(*state);
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum State {
    #[default]
    None,
    Direct(u128),
    Inherit(PlotState)
}

impl State {
    pub fn apply(&self, plot: &mut Plot) {
        match self {
            Self::Inherit(state) => state.apply(plot),
            _ => ()
        }
    }
}