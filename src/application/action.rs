use crate::{simulator::*, config, project::ProjectRef, renderer::{vector::Vector2, Color}, id::Id};

use super::*;

#[derive(Default)]
pub struct ActionStack {
    actions: Vec<Action>,
    next: usize,
    dirty: bool
}

impl ActionStack {
    pub fn undo(&mut self, app: &Application) {
        if self.next > 0 && let Some(action) = self.actions.get(self.next - 1) {
            self.next -= 1;
            self.dirty = true;
            self.update_buttons(&app.imp().undo_button(), &app.imp().redo_button());

            info!("Un-doing action {}", self.next);
            action.undo(app);
        }
    }

    pub fn redo(&mut self, app: &Application) {
        if let Some(action) = self.actions.get_mut(self.next) {
            self.next += 1;
            self.dirty = true;
            
            info!("Re-doing action {}", self.next - 1);
            action.exec(app);

            self.update_buttons(&app.imp().undo_button(), &app.imp().redo_button());
        }
    }

    fn update_buttons(&self, undo_button: &gtk::Button, redo_button: &gtk::Button) {
        undo_button.set_sensitive(self.next != 0);
        redo_button.set_sensitive(self.actions.get(self.next).is_some());
    }

    pub fn add(&mut self, app: &Application, mut action: Action) {
        while self.actions.get(self.next).is_some() {
            self.actions.pop();
        }

        action.exec(app);
        
        self.next += 1;
        self.dirty = true;
        self.actions.push(action);
        self.update_buttons(&app.imp().undo_button(), &app.imp().redo_button());

        while self.actions.len() > config::MAX_ACTION_STACK_SIZE {
            self.actions.remove(0); // shorten the action stack to prevent memory leaks
        }
    }

    pub fn reset(&mut self) {
        self.next = 0;
        self.dirty = false;
        self.actions.clear();
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty
    }
}

pub enum Action {
    NewBlock(PlotProvider, Block),
    PasteBlocks(PlotProvider, Vec<Block>, Vec<Connection>),
    MoveBlock(PlotProvider, BlockID, Vector2<i32>, Vector2<i32>),
    MoveWaypoint(PlotProvider, SegmentID, Vector2<i32>, Vector2<i32>),
    NewConnection(PlotProvider, Connection),
    WaypointToConnection(PlotProvider, SegmentID, Segment, BlockID, u8),
    AddSegment(PlotProvider, SegmentID, Segment, Option<Id>),
    ChangeBorderColor(PlotProvider, Color, Vec<BlockID>, Vec<Option<Color>>),
    DeleteSelection(PlotProvider, Vec<Block>, Vec<Connection>, Vec<(SegmentID, Segment)>),
    CreateModule(ProjectRef, Module),
    DeleteModule(ProjectRef, Module),
}

impl Action {
    fn exec(&mut self, app: &Application) {
        match self {
            Self::NewBlock(plot_provider, block) => { // place a new block
                plot_provider.with_mut(|plot| plot.add_block(block.clone()));
                app.imp().rerender_editor();
            }
            Self::PasteBlocks(plot_provier, blocks, connections) => {
                plot_provier.with_mut(|plot| {
                    blocks.iter().for_each(|block| plot.add_block(block.clone()));
                    connections.iter().for_each(|connection| plot.add_connection(connection.clone()));
                });
                app.imp().rerender_editor();
            }
            Self::MoveBlock(plot_provider, block_id, _from, to) => {
                plot_provider.with_mut(|plot| if let Some(block) = plot.get_block_mut(*block_id) {
                    block.set_position(*to);
                });
                app.imp().rerender_editor();
            }
            Self::MoveWaypoint(plot_provider, segment_id, _from, to) => {
                plot_provider.with_mut(|plot| 
                    if let Some(waypoint) = plot.get_connection_mut(segment_id.connection_id()).and_then(|c| c.get_segment_mut(segment_id.location())) {
                        waypoint.set_position(*to);
                    }
                );
                app.imp().rerender_editor();
            }
            Self::NewConnection(plot_provider, connection) => {
                plot_provider.with_mut(|plot| {
                    plot.add_connection(connection.clone());
                });
                app.imp().rerender_editor();
            }
            Self::WaypointToConnection(plot_provider, segment_id, _segment, block_id, block_port) => {
                plot_provider.with_mut(|plot|
                    if let Some(waypoint) = plot.get_connection_mut(segment_id.connection_id()).and_then(|c| c.get_segment_mut(segment_id.location())) {
                        waypoint.convert(*block_id, *block_port);

                        if let Some(block) = plot.get_block_mut(*block_id) {
                            block.set_connection(Connector::Input(*block_port), Some(*segment_id.connection_id()));
                            plot.add_block_to_update(*block_id);
                        }
                    }
                );
                app.imp().rerender_editor();
            }
            Self::AddSegment(plot_provider, segment_id, segment, index) => {
                *index = plot_provider.with_mut(|plot|
                    if let Some(root) = plot.get_connection_mut(segment_id.connection_id()).and_then(|c| c.get_segment_mut(segment_id.location())) {
                        let index = root.add_segment(segment.clone());

                        if let Segment::Block(block_id, port) = *segment && let Some(block) = plot.get_block_mut(block_id) {
                            block.set_connection(Connector::Input(port), Some(*segment_id.connection_id()));
                            plot.add_block_to_update(block_id);
                        }

                        index
                    }
                    else {
                        None
                    }
                ).flatten();
                app.imp().rerender_editor();
            }
            Self::ChangeBorderColor(plot_provider, new_color, block_ids, old_colors) => {
                let old = plot_provider.with_mut(|plot| {
                    block_ids.iter().map(|block_id| {
                        plot.get_block_mut(*block_id).map(|block| {
                            let old_color = *block.color();
                            block.set_color(Some(*new_color));
                            old_color
                        })
                    }).flatten().collect()
                });
                if let Some(old) = old {
                    *old_colors = old;
                }

                app.imp().rerender_editor();
            }
            Self::DeleteSelection(plot_provider, blocks, incoming_connections, segments) => {
                let connections = plot_provider.with_mut(|plot| {
                    let mut connections = vec![];
                    for block in blocks.iter() {
                        connections.append(&mut plot.delete_block(block.id()))
                    }
                    connections
                }).unwrap_or_default();

                blocks.iter_mut().for_each(|block| block.set_highlighted(false));

                *incoming_connections = connections;
                app.imp().rerender_editor();
            }
            Self::CreateModule(project, module) => {
                if let Some(window) = app.imp().window().borrow().as_ref() {
                    window.add_module_to_ui(app, &module);
                }
                project.lock().unwrap().add_module(module.clone());
            }
            Self::DeleteModule(project, module) => {
                if let Some(window) = app.imp().window().borrow().as_ref() {
                    window.remove_module_from_ui(module.name());
                }
                project.lock().unwrap().remove_module(module.name());
            }
        }
    }

    fn undo(&self, app: &Application) {
        match self {
            Self::NewBlock(plot_provider, block) => { // remove a block
                plot_provider.with_mut(|plot| plot.delete_block(block.id()));
                app.imp().rerender_editor();
            }
            Self::PasteBlocks(plot_provier, blocks, connections) => {
                plot_provier.with_mut(|plot|  {
                    blocks.iter().for_each(|block| { 
                        plot.delete_block(block.id());
                    });
                    connections.iter().for_each(|connection| {
                        plot.remove_connection(connection.id());
                    })
                });
                app.imp().rerender_editor();
            }
            Self::MoveBlock(plot_provider, block_id, from, _to) => {
                plot_provider.with_mut(|plot| 
                    if let Some(block) = plot.get_block_mut(*block_id) {
                        block.set_position(*from);
                    }
                );
                app.imp().rerender_editor();
            }
            Self::MoveWaypoint(plot_provider, segment_id, from, _to) => {
                plot_provider.with_mut(|plot| 
                    if let Some(waypoint) = plot.get_connection_mut(segment_id.connection_id()).and_then(|c| c.get_segment_mut(segment_id.location())) {
                        waypoint.set_position(*from);
                    }
                );
                app.imp().rerender_editor();
            }
            Self::NewConnection(plot_provider, connection) => {
                plot_provider.with_mut(|plot| {
                    plot.remove_connection(connection.id());
                });
                app.imp().rerender_editor();
            }
            Self::WaypointToConnection(plot_provider, segment_id, segment, block_id, block_port) => {
                plot_provider.with_mut(|plot| {
                    if let Some(waypoint) = plot.get_connection_mut(segment_id.connection_id()).and_then(|c| c.get_segment_mut(segment_id.location())) {
                        *waypoint = segment.clone();
                    }

                    if let Some(block) = plot.get_block_mut(*block_id) {
                        block.set_connection(Connector::Input(*block_port), None);
                        plot.add_block_to_update(*block_id);
                    }
                });
                app.imp().rerender_editor();
            }
            Self::AddSegment(plot_provider, segment_id, segment, id) => {
                plot_provider.with_mut(|plot| {
                    if let Some(root) = plot.get_connection_mut(segment_id.connection_id()).and_then(|c| c.get_segment_mut(segment_id.location())) &&
                        let Some(id) = id {
                        root.remove_segment(id);
                    }

                    if let Segment::Block(block_id, port) = *segment && let Some(block) = plot.get_block_mut(block_id) {
                        block.set_connection(Connector::Input(port), None);
                        plot.add_block_to_update(block_id);
                    }
                });
                app.imp().rerender_editor();
            }
            Self::ChangeBorderColor(plot_provider, _new_color, block_ids, old_colors) => {
                plot_provider.with_mut(|plot| {
                    block_ids.iter().zip(old_colors).for_each(|(block_id, old_color)| {
                        if let Some(block) = plot.get_block_mut(*block_id) {
                            block.set_color(*old_color);
                        }
                    });
                });

                app.imp().rerender_editor();
            }
            Self::DeleteSelection(plot_provider, blocks, incoming_connections, segments) => {
                plot_provider.with_mut(|plot| {
                    blocks.iter().for_each(|block| plot.add_block(block.clone()));
                    incoming_connections.iter().for_each(|connection| plot.add_connection(connection.clone()));
                });
                app.imp().rerender_editor();
            }
            Self::CreateModule(project, module) => {
                if let Some(window) = app.imp().window().borrow().as_ref() {
                    window.remove_module_from_ui(module.name());
                }
                project.lock().unwrap().remove_module(module.name());
            }
            Self::DeleteModule(project, module) => {
                if let Some(window) = app.imp().window().borrow().as_ref() {
                    window.add_module_to_ui(app, &module);
                }
                project.lock().unwrap().add_module(module.clone());
            }
        }
    }
}
