use super::{Block, BlockID, Connection, ConnectionID, Port, Identifiable};
use crate::{renderer::*, selection::*, project::{ProjectRef, Project}};
use std::{collections::{HashMap, HashSet}, cmp};
use serde::{Serialize, Deserialize};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum PlotDescriptor {
    Main(),
    Module(String)
}

impl From<&PlotProvider> for PlotDescriptor {
    fn from(value: &PlotProvider) -> Self {
        match value {
            PlotProvider::Main(_) => Self::Main(),
            PlotProvider::Module(_, name) => Self::Module(name.clone()),
            _ => panic!()
        }
    }
}

#[derive(Clone, Default)]
pub enum PlotProvider {
    #[default]
    None,
    Main(ProjectRef),
    Module(ProjectRef, String),
}

impl Into<PlotDescriptor> for PlotProvider {
    fn into(self) -> PlotDescriptor {
        match self {
            Self::None => panic!(),
            Self::Main(_) => PlotDescriptor::Main(),
            Self::Module(_, name) => PlotDescriptor::Module(name)
        }
    }
}

impl PlotProvider {
    #[inline]
    pub fn with<T>(&self, func: impl Fn(&Plot) -> T) -> Option<T> {
        match self {
            Self::None => None,
            Self::Main(project) => Some(func(project.lock().unwrap().main_plot())),
            Self::Module(project, module) => project
                .lock()
                .unwrap()
                .plot(module)
                .map(|plot| func(plot))
        }
    }

    #[inline]
    pub fn with_mut<T>(&self, func: impl Fn(&mut Plot) -> T) -> Option<T> {
        match self {
            Self::None => None,
            Self::Main(project) => Some(func(project.lock().unwrap().main_plot_mut())),
            Self::Module(project, module) => project
                .lock()
                .unwrap()
                .plot_mut(module)
                .map(|plot| func(plot)),
        }
    }

    pub fn project(&self) -> Option<ProjectRef> {
        match self {
            Self::Main(project) | 
            Self::Module(project, _) => Some(project.clone()),
            _ => None
        }
    }

    pub fn is_main(&self) -> bool {
        matches!(self, PlotProvider::Main(_))
    }

    pub fn is_module(&self) -> Option<&String> {
        match self {
            Self::Module(_, name) => Some(name),
            _ => None
        }
    }
}

#[derive(Serialize, Debug, Default, Deserialize, Clone)]
pub struct Plot {
    blocks: HashMap<BlockID, Block>,
    connections: HashMap<ConnectionID, Connection>,

    #[serde(skip)]
    selection: Selection,

    #[serde(skip)]
    to_update: HashSet<BlockID>
}

impl Identifiable for Plot {
    type ID = PlotDescriptor;
}

impl Plot {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            connections: HashMap::new(),
            selection: Selection::None,
            to_update: HashSet::new()
        }
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

    pub fn get_connection(&self, id: ConnectionID) -> Option<&Connection> {
        self.connections.get(&id)
    }

    pub fn add_connection(&mut self, connection: Connection) {
        let origin = self.blocks.get_mut(&connection.from().block_id).expect("faulty origin block");
        origin.set_connection(connection.to_port(), Some(connection.id()));

        let destination = self.blocks.get_mut(&connection.to().block_id).expect("faulty destination block");
        destination.set_connection(connection.from_port(), Some(connection.id()));

        self.to_update.insert(connection.origin_id());
        self.connections.insert(connection.id(), connection);
    }

    pub fn remove_connection(&mut self, id: ConnectionID) -> Option<Connection> {
        if let Some(c) = self.connections.get(&id) {
            let connection = c.clone();
            let refactor = |plot: &mut Plot, id: BlockID, port: Port| {
                if let Some(block) = plot.get_block_mut(id) {
                    block.set_connection(port, None);
                }
                plot.to_update.insert(id);
            };

            refactor(self, connection.destination_id(), connection.to_port());
            refactor(self, connection.origin_id(), connection.from_port());

            self.connections.remove(&id);
            return Some(connection);
        }
        None
    }

    pub fn delete_block(&mut self, id: BlockID) -> Vec<Connection> {
        if let Some(block) = self.blocks.get(&id) && block.unique() {
            return vec![];
        }

        let mut deleted_connections = vec![];
        if let Some(block) = self.blocks.get(&id) {
            deleted_connections = block.connected_to()
                .iter()
                .filter_map(|id| self.remove_connection(*id))
                .collect();
            self.blocks.remove(&id);
        }

        deleted_connections
    }

    pub fn add_block_to_update(&mut self, block: BlockID) {
        self.to_update.insert(block);
    }

    pub fn to_update(&self) -> &HashSet<BlockID> {
        &self.to_update
    }

    pub fn to_update_mut(&mut self) -> &mut HashSet<BlockID> {
        &mut self.to_update
    }

    pub fn simulate(&mut self, project: &mut Project) -> bool {
        let mut updated = HashSet::new();
        let mut changes = false;

        loop {
            if self.to_update.len() == 0 {
                break;
            }
            let to_update = self.to_update.clone();
            self.to_update.clear();
            changes = true;
            
            to_update.iter().for_each(|block_id|
                if let Some(block) = self.blocks.get_mut(block_id) {
                    block.simulate(&mut self.connections, &mut self.to_update, project);
                    updated.insert(*block_id);   
                }
            );


            // TODO: check for recursion
        }

        changes
    }
}

impl Renderable for Plot {
    fn render<R>(&self, renderer: &R, plot: &Plot) -> Result<(), R::Error>
        where R: Renderer
    {
        let screen_space = renderer.screen_space();

        // render all blocks
        for (_, block) in self.blocks.iter().filter(|(_, block)| block.is_in_area(screen_space)) {
            block.render(renderer, plot)?;
        }

        // render all connections
        renderer.set_line_width(4.);
        for (_, connection) in &self.connections {
            connection.render(renderer, plot)?;
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
            Selection::Single(id, _) => {
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
            Selection::Single(id, _) => vec![id],
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
