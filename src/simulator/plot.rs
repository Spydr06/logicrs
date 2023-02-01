use super::{Block, BlockID, Connection};
use crate::{renderer::*, selection::*, project::ProjectRef};
use std::{collections::HashMap, cmp};
use serde::{Serialize, Deserialize};

#[derive(Clone)]
pub enum PlotProvider {
    None,
    Main(ProjectRef),
    Module(ProjectRef, String),
}

impl Default for PlotProvider {
    fn default() -> Self {
        Self::None
    }
}

impl PlotProvider {
    #[inline]
    pub fn with<T>(&self, func: impl Fn(&Plot) -> T) -> Option<T> {
        match self {
            Self::Main(project) => Some(func(project.lock().unwrap().main_plot())),
            Self::Module(project, module) => project
                .lock()
                .unwrap()
                .plot(module)
                .map(|plot| func(plot)),
            Self::None => None
        }
    }

    #[inline]
    pub fn with_mut<T>(&self, func: impl Fn(&mut Plot) -> T) -> Option<T> {
        match self {
            Self::Main(project) => Some(func(project.lock().unwrap().main_plot_mut())),
            Self::Module(project, module) => project
                .lock()
                .unwrap()
                .plot_mut(module)
                .map(|plot| func(plot)),
            Self::None => None
        }
    }
}

#[derive(Serialize, Debug, Default, Deserialize, Clone)]
pub struct Plot {
    blocks: HashMap<BlockID, Block>,
    id_counter: BlockID,

    #[serde(skip)]
    selection: Selection,
}

impl Plot {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            selection: Selection::None,
            id_counter: 0
        }
    }

    pub fn next_id(&mut self) -> BlockID {
        self.id_counter += 1;
        self.id_counter
    }

    pub fn current_id(&self) -> BlockID {
        self.id_counter
    }

    pub fn blocks(&self) -> &HashMap<BlockID, Block> {
        &self.blocks
    }

    pub fn blocks_mut(&mut self) -> &mut HashMap<BlockID, Block> {
        &mut self.blocks
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.insert(block.id(), block);
    }

    pub fn get_block(&self, id: BlockID) -> Option<&Block> {
        self.blocks.get(&id)
    }

    pub fn get_block_mut(&mut self, id: BlockID) -> Option<&mut Block> {
        self.blocks.get_mut(&id)
    }

    pub fn get_block_at(&self, position: (i32, i32)) -> Option<BlockID> {
        for (i, block) in self.blocks.iter() {
            if block.touches(position) {
                return Some(*i);
            }
        }

        None
    }

    pub fn delete_block(&mut self, id: BlockID) -> Vec<Connection> {
        if let Some(block) = self.blocks.get(&id) && !block.deleteable() {
            return vec![];
        }

        let mut deleted_connections = vec![];
        self.blocks.values_mut().for_each(|block| {
                let connections = block.connections_mut().iter_mut().filter(
                    |c| c.as_ref().map(|c| c.contains(id)
                ).unwrap_or(false));

                let mut vec: Vec<Connection> = connections.map(|c| c.as_ref().unwrap().clone()).collect();
                deleted_connections.append(&mut vec);

                block.connections_mut().iter_mut().filter(
                    |c| c.as_ref().map(|c| c.contains(id)
                ).unwrap_or(false)).for_each(|c| *c = None);
            }
        );

        self.blocks.remove(&id);
        deleted_connections
    }
}

impl Renderable for Plot {
    fn render<R>(&self, renderer: &R, plot: &Plot) -> Result<(), R::Error>
        where R: Renderer
    {
        let screen_space = renderer.screen_space();

        // render grid
        renderer.set_line_width(4.);
        for (_, block) in self.blocks() {
            for c in block.connections().iter().filter(|c| c.is_some()) {
                c.as_ref().unwrap().render(renderer, plot)?;
            }
        }
        
        // render all blocks
        for (_, block) in self.blocks().iter().filter(|(_, block)| block.is_in_area(screen_space)) {
            block.render(renderer, plot)?;
        }

        Ok(())
    }
}

impl SelectionField for Plot {
    fn selection(&self) -> &Selection {
        &self.selection
    }

    fn set_selection(&mut self, selection: Selection) {
        self.selection = selection;
    }

    fn unhighlight(&mut self) {
        match self.selection.clone() {
            Selection::Single(id) => {
                self.get_block_mut(id).map(|b| b.set_highlighted(false));
            },
            Selection::Many(ids) => {
                ids.iter().for_each(|id| {
                    self.get_block_mut(*id).map(|b| b.set_highlighted(false));
                });
            },
            Selection::Area(_, _) => self.blocks_mut().iter_mut().for_each(|(_, v)| v.set_highlighted(false)),
            _ => ()
        }

        self.selection = Selection::None
    }

    fn selected(&self) -> Vec<BlockID> {
        match self.selection.clone() {
            Selection::Single(id) => vec![id],
            Selection::Many(ids) => ids,
            _ => vec![]
        }
    }

    fn highlight_area(&mut self) {
        if let Selection::Area(selection_start, selection_end) = self.selection {
            let mut selected = Vec::new();

            let x1 = cmp::min(selection_start.0, selection_end.0);
            let y1 = cmp::min(selection_start.1, selection_end.1);
            let x2 = cmp::max(selection_start.0, selection_end.0);
            let y2 = cmp::max(selection_start.1, selection_end.1);
            
            for (_, block) in self.blocks_mut().iter_mut() {
                if block.is_in_area(((x1, y1), (x2, y2))) {
                    block.set_highlighted(true);
                    selected.push(block.id());
                }
            }

            self.selection = Selection::Many(selected)
        }
    }

    fn select_all(&mut self) {
        self.selection = Selection::Many(self.blocks.keys().map(|id| *id).collect());
        self.blocks.iter_mut().for_each(|(_, block)| block.set_highlighted(true));
    }
}