use super::Block;
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
    pub fn with(&self, func: impl Fn(&Plot)) {
        match self {
            Self::Main(project) => func(project.lock().unwrap().main_plot()),
            Self::Module(project, module) => 
                if let Some(plot) = project.lock().unwrap().plot(module) {
                    func(plot);
                },
            Self::None => {}
        };
    }

    #[inline]
    pub fn with_mut(&self, func: impl Fn(&mut Plot)) {
        match self {
            Self::Main(project) => func(project.lock().unwrap().main_plot_mut()),
            Self::Module(project, module) => {
                if let Some(plot) = project.lock().unwrap().plot_mut(module) {
                    func(plot);
                };
            }
            Self::None => {}
        };
    }
}

#[derive(Serialize, Debug, Default, Deserialize)]
pub struct Plot {
    blocks: HashMap<u32, Block>,
    selection: Selection
}

impl Plot {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            selection: Selection::None
        }
    }

    pub fn blocks(&self) -> &HashMap<u32, Block> {
        &self.blocks
    }

    pub fn blocks_mut(&mut self) -> &mut HashMap<u32, Block> {
        &mut self.blocks
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.insert(block.id(), block);
    }

    pub fn get_block(&self, id: u32) -> Option<&Block> {
        self.blocks.get(&id)
    }

    pub fn get_block_mut(&mut self, id: u32) -> Option<&mut Block> {
        self.blocks.get_mut(&id)
    }

    pub fn get_block_at(&self, position: (i32, i32)) -> Option<u32> {
        for (i, block) in self.blocks.iter() {
            if block.touches(position) {
                return Some(*i);
            }
        }

        None
    }

    pub fn delete_block(&mut self, id: u32) {
        info!("Remove block {id}");

        self.blocks.values_mut().for_each(|block| 
            block.connections_mut().iter_mut().filter(
                |c| c.as_ref().map(|c| c.contains(id)
            ).unwrap_or(false)).for_each(|c| *c = None)
        );

        self.blocks.remove(&id);
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
            for c in block.connections() {
                if let Some(connection) = c {
                    connection.render(renderer, plot)?;
                }
            }
        }
        
        // render all blocks
        for (_, block) in self.blocks().iter().filter(|(_, block)| block.is_in_area(screen_space)) {
            block.render(renderer, plot)?
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
            Selection::Connection { block_id: _, output: _, start: _, position: _ } => (),
            Selection::None => ()
        }

        self.selection = Selection::None
    }

    fn delete_selected(&mut self) {
        match self.selection.clone() {
            Selection::Single(id) => self.delete_block(id),
            Selection::Many(ids) => ids.iter().for_each(|id| self.delete_block(*id)),
            Selection::Area(_, _) => {}
            Selection::None | Selection::Connection {block_id: _, output: _, start: _, position: _} => {},
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
                if block.is_in_area((x1, y1, x2, y2)) {
                    block.set_highlighted(true);
                    selected.push(block.id());
                }
            }

            self.selection = Selection::Many(selected)
        }
    }
}