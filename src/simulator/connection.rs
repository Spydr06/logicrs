use super::*;
use crate::{
    application::editor::{self, EditorMode},
    id::Id,
    renderer::{vector::*, *},
};
use serde::{Deserialize, Serialize};
use std::f64;

pub type ConnectionID = Id;

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum Port {
    Input(BlockID, u8),
    Output(BlockID, u8),
}

impl Port {
    pub fn index(&self) -> u8 {
        match self {
            Self::Input(_, index) | Self::Output(_, index) => *index,
        }
    }

    pub fn block_id(&self) -> BlockID {
        match self {
            Self::Input(id, _) | Self::Output(id, _) => *id,
        }
    }

    pub fn set_block_id(&mut self, block_id: BlockID) {
        match self {
            Self::Input(id, _) | Self::Output(id, _) => *id = block_id,
        }
    }
}

impl From<Port> for Connector {
    fn from(value: Port) -> Self {
        match value {
            Port::Input(_, port) => Self::Input(port),
            Port::Output(_, port) => Self::Output(port),
        }
    }
}

pub type SegmentLocation = Vec<Id>;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SegmentID {
    connection_id: ConnectionID,
    location: SegmentLocation,
}

impl SegmentID {
    fn new(connection_id: ConnectionID, location: Vec<Id>) -> Self {
        Self {
            connection_id,
            location,
        }
    }

    pub fn connection_id(&self) -> &ConnectionID {
        &self.connection_id
    }

    pub fn location(&self) -> &SegmentLocation {
        &self.location
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Segment {
    Block(BlockID, u8),
    Waypoint(HashMap<Id, Segment>, Vector2<i32>, bool),
}

impl Segment {
    pub const HITBOX_SIZE: i32 = editor::GRID_SIZE / 2;

    pub fn position(&self) -> Option<&Vector2<i32>> {
        match self {
            Self::Waypoint(_, position, _) => Some(position),
            _ => None,
        }
    }

    pub fn set_position(&mut self, new_position: Vector2<i32>) {
        if let Self::Waypoint(_, position, _) = self {
            *position = new_position;
        }
    }

    pub fn set_highlighted(&mut self, highlighted: bool) {
        if let Self::Waypoint(.., self_highlighted) = self {
            *self_highlighted = highlighted;
        }
    }

    pub fn is_in_area(&mut self, area: &ScreenSpace) -> bool {
        if let Self::Waypoint(_, position, _) = self {
            let hs = Vector2::new(Self::HITBOX_SIZE, Self::HITBOX_SIZE);
            !(Vector2::cast(*position - hs) > area.1 || Vector2::cast(*position + hs) < area.0)
        } else {
            false
        }
    }

    pub fn add_segment(&mut self, segment: Segment) -> Option<Id> {
        if let Self::Waypoint(segments, ..) = self {
            let id = Id::new();
            segments.insert(id, segment);
            Some(id)
        } else {
            None
        }
    }

    pub fn remove_segment(&mut self, id: &Id) {
        if let Self::Waypoint(segments, ..) = self {
            segments.remove(id);
        }
    }

    pub fn convert(&mut self, block_id: BlockID, port: u8) {
        *self = Self::Block(block_id, port)
    }

    fn render<R>(
        &self,
        active: bool,
        start: Vector2<i32>,
        renderer: &R,
        plot: &Plot,
    ) -> Result<(), R::Error>
    where
        R: Renderer,
    {
        match self {
            Self::Block(block_id, port) => {
                let end_block = plot.get_block(*block_id).unwrap();
                let end = end_block.get_connector_pos(Connector::Input(*port));
                render_line(active, start, end, renderer)?;
                render_block_connector(end, active, end_block.highlighted(), renderer)
            }
            Self::Waypoint(segments, position, highlighted) => {
                render_line(active, start, *position, renderer)?;

                for segment in segments.values() {
                    segment.render(active, *position, renderer, plot)?;
                }

                render_waypoint(*position, active, *highlighted, renderer)
            }
        }
    }

    fn destinations(&self, ports: &mut Vec<Port>) {
        match self {
            Self::Block(block_id, port) => ports.push(Port::Input(*block_id, *port)),
            Self::Waypoint(segments, ..) => segments
                .iter()
                .for_each(|(_, segment)| segment.destinations(ports)),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::Waypoint(segments, ..) => segments.is_empty(),
            _ => false,
        }
    }

    fn remove_unselected_branches(&mut self, selection: &Vec<BlockID>) -> bool {
        match self {
            Self::Block(block_id, ..) => !selection.contains(block_id),
            Self::Waypoint(segments, ..) => {
                segments.retain(|_, segment| !segment.remove_unselected_branches(selection));
                self.is_empty()
            }
        }
    }

    fn refactor_id(&mut self, old_id: BlockID, new_id: BlockID) {
        match self {
            Self::Block(block_id, ..) if *block_id == old_id => *block_id = new_id,
            Self::Waypoint(segments, ..) => segments
                .iter_mut()
                .for_each(|(_, segment)| segment.refactor_id(old_id, new_id)),
            _ => (),
        }
    }

    fn touches(&self, point: Vector2<i32>) -> bool {
        match self {
            Self::Waypoint(_, position, _) => {
                point.0 > position.0 - Self::HITBOX_SIZE
                    && point.0 < position.0 + Self::HITBOX_SIZE
                    && point.1 > position.1 - Self::HITBOX_SIZE
                    && point.1 < position.1 + Self::HITBOX_SIZE
            }
            _ => false,
        }
    }

    fn waypoint_at(&self, point: Vector2<i32>, location: &mut SegmentLocation) -> bool {
        match self {
            Self::Waypoint(..) if self.touches(point) => true,
            Self::Waypoint(segments, ..) => segments.iter().any(|(id, segment)| {
                location.push(*id);
                let result = segment.waypoint_at(point, location);
                if !result {
                    location.pop();
                }
                result
            }),
            _ => false,
        }
    }

    fn get_segment_mut(
        &mut self,
        location: &SegmentLocation,
        depth: &mut usize,
    ) -> Option<&mut Segment> {
        let mut_ref_ptr = self as *mut _;
        match self {
            Self::Waypoint(segments, ..) => {
                *depth += 1;
                if let Some(next) = location.get(*depth) {
                    segments
                        .get_mut(next)
                        .and_then(|segment| segment.get_segment_mut(location, depth))
                } else {
                    Some(unsafe { &mut *mut_ref_ptr })
                }
            }
            _ => Some(unsafe { &mut *mut_ref_ptr }),
        }
    }

    fn get_segment(&self, location: &SegmentLocation, depth: &mut usize) -> Option<&Segment> {
        match self {
            Self::Waypoint(segments, ..) => {
                *depth += 1;
                if let Some(next) = location.get(*depth) {
                    segments
                        .get(next)
                        .and_then(|segment| segment.get_segment(location, depth))
                } else {
                    Some(self)
                }
            }
            _ => Some(self),
        }
    }

    fn for_each_mut_segment<F>(&mut self, func: &F)
    where
        F: Fn(&mut Segment),
    {
        func(self);
        if let Self::Waypoint(segments, ..) = self {
            for segment in segments.values_mut() {
                segment.for_each_mut_segment(func);
            }
        }
    }

    fn for_each_mut_segment_id<F>(&mut self, func: &mut F, waypoint_id: &mut SegmentID)
    where
        F: FnMut(&mut Segment, &SegmentID),
    {
        let mut_ref_ptr = self as *mut _;
        if let Self::Waypoint(segments, ..) = self {
            func(unsafe { &mut *mut_ref_ptr }, waypoint_id);
            for (id, segment) in segments.iter_mut() {
                waypoint_id.location.push(*id);
                segment.for_each_mut_segment_id(func, waypoint_id);
                waypoint_id.location.pop();
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Connection {
    id: ConnectionID,
    active: bool,
    origin: Port,
    segments: HashMap<Id, Segment>,
}

impl Identifiable for Connection {
    type ID = ConnectionID;
}

impl Connection {
    pub fn new(origin: Port, segments: Vec<Segment>) -> Self {
        Self {
            id: Id::new(),
            active: false,
            origin,
            segments: segments
                .into_iter()
                .map(|segment| (Id::new(), segment))
                .collect(),
        }
    }

    pub fn new_basic(
        origin_block: BlockID,
        origin_port: u8,
        destination_block: BlockID,
        destination_port: u8,
    ) -> Self {
        Self {
            id: Id::new(),
            active: false,
            origin: Port::Output(origin_block, origin_port),
            segments: {
                let mut segments = HashMap::new();
                segments.insert(
                    Id::new(),
                    Segment::Block(destination_block, destination_port),
                );
                segments
            },
        }
    }

    pub fn id(&self) -> ConnectionID {
        self.id
    }

    pub fn set_id(&mut self, id: ConnectionID) {
        self.id = id;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, is_active: bool) {
        self.active = is_active;
    }

    pub fn origin(&self) -> Port {
        self.origin
    }

    pub fn mut_origin(&mut self) -> &mut Port {
        &mut self.origin
    }

    pub fn destinations(&self) -> Vec<Port> {
        let mut ports = vec![];
        self.segments
            .iter()
            .for_each(|(_, segment)| segment.destinations(&mut ports));
        ports
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn remove_unselected_branches(&mut self, selected: &Vec<Id>) -> bool {
        self.segments
            .retain(|_, segment| !segment.remove_unselected_branches(selected));
        self.segments.is_empty()
    }

    pub fn refactor_id(&mut self, old_id: BlockID, new_id: BlockID) {
        if self.origin.block_id() == old_id {
            self.origin.set_block_id(new_id)
        } else {
            self.segments
                .iter_mut()
                .for_each(|(_, segment)| segment.refactor_id(old_id, new_id))
        }
    }

    pub fn waypoint_at(&self, position: Vector2<i32>) -> Option<SegmentID> {
        let mut location = vec![Id::empty()];
        self.segments
            .iter()
            .any(|(id, segment)| {
                location[0] = *id;
                segment.waypoint_at(position, &mut location)
            })
            .then_some(SegmentID::new(self.id, location))
    }

    pub fn add_segment(&mut self, segment: Segment) {
        self.segments.insert(Id::new(), segment);
    }

    pub fn segments(&self) -> &HashMap<Id, Segment> {
        &self.segments
    }

    pub fn get_segment(&self, location: &SegmentLocation) -> Option<&Segment> {
        if location.is_empty() {
            return None;
        }

        let mut i = 0;
        self.segments
            .get(&location[i])
            .and_then(|segment| segment.get_segment(location, &mut i))
    }

    pub fn get_segment_mut(&mut self, location: &SegmentLocation) -> Option<&mut Segment> {
        if location.is_empty() {
            return None;
        }

        let mut i = 0;
        self.segments
            .get_mut(&location[i])
            .and_then(|segment| segment.get_segment_mut(location, &mut i))
    }

    pub fn for_each_mut_segment<F>(&mut self, func: F)
    where
        F: Fn(&mut Segment),
    {
        for segment in self.segments.values_mut() {
            segment.for_each_mut_segment(&func);
        }
    }

    pub fn for_each_mut_segment_id<F>(&mut self, mut func: F)
    where
        F: FnMut(&mut Segment, &SegmentID),
    {
        let mut waypoint_id = SegmentID::new(self.id, vec![Id::empty()]);
        for (id, segment) in self.segments.iter_mut() {
            waypoint_id.location[0] = *id;
            segment.for_each_mut_segment_id(&mut func, &mut waypoint_id)
        }
    }
}

fn render_waypoint<R>(
    position: Vector2<i32>,
    active: bool,
    highlighted: bool,
    renderer: &R,
) -> Result<(), R::Error>
where
    R: Renderer,
{
    let connector_color = unsafe {
        if active {
            &COLOR_THEME.enabled_fg_color
        } else {
            &COLOR_THEME.disabled_fg_color
        }
    };
    renderer
        .set_line_width(1.)
        .arc(position, 6., 0., f64::consts::TAU)
        .set_color(connector_color)
        .fill_preserve()?
        .set_color(unsafe {
            if highlighted {
                &COLOR_THEME.accent_fg_color
            } else {
                &COLOR_THEME.border_color
            }
        })
        .stroke()
        .map(|_| ())
}

pub fn render_block_connector<R>(
    position: Vector2<i32>,
    active: bool,
    highlighted: bool,
    renderer: &R,
) -> Result<(), R::Error>
where
    R: Renderer,
{
    let connector_color = unsafe {
        if active {
            &COLOR_THEME.enabled_fg_color
        } else {
            &COLOR_THEME.disabled_fg_color
        }
    };
    renderer
        .set_line_width(1.)
        .arc(position, 6., 0., f64::consts::TAU)
        .set_color(connector_color)
        .fill_preserve()?
        .set_color(unsafe {
            if highlighted {
                &COLOR_THEME.accent_fg_color
            } else {
                &COLOR_THEME.border_color
            }
        })
        .stroke()
        .map(|_| ())
}

pub fn render_line<R>(
    active: bool,
    start: Vector2<i32>,
    end: Vector2<i32>,
    renderer: &R,
) -> Result<(), R::Error>
where
    R: Renderer,
{
    renderer
        .set_color(unsafe {
            if active {
                &COLOR_THEME.enabled_bg_color
            } else {
                &COLOR_THEME.disabled_bg_color
            }
        })
        .set_line_width(4.);

    match renderer.editor_mode() {
        EditorMode::Normal => {
            let offset = (
                Vector2(start.0 + ((end.0 - start.0) as f32 * 0.7) as i32, start.1),
                Vector2(end.0 + ((start.0 - end.0) as f32 * 0.7) as i32, end.1),
            );
            renderer
                .move_to(start)
                .curve_to(offset.0, offset.1, end)
                .stroke()
        }
        EditorMode::Grid => renderer.move_to(start).line_to(end).stroke(),
    }
    .map(|_| ())
}

impl Renderable for Connection {
    fn render<R>(&self, renderer: &R, plot: &Plot) -> Result<(), R::Error>
    where
        R: Renderer,
    {
        let origin_block = plot.get_block(self.origin.block_id());
        if origin_block.is_none() {
            return Ok(());
        }
        let origin_block = origin_block.unwrap();
        let origin_pos = origin_block.get_connector_pos(self.origin.into());

        for segment in self.segments.values() {
            segment.render(self.active, origin_pos, renderer, plot)?
        }

        render_block_connector(
            origin_pos,
            self.active,
            origin_block.highlighted(),
            renderer,
        )
    }
}
