use crate::renderer::{
    vector::{Vector2, VectorCast},
    Renderable, COLOR_THEME,
};

#[derive(Default, Copy, Clone)]
pub enum EditorMode {
    #[default]
    Normal,
    Grid,
}

impl From<bool> for EditorMode {
    fn from(b: bool) -> Self {
        match b {
            true => Self::Grid,
            false => Self::Normal,
        }
    }
}

// grid size in pixels at scale 1.0
pub const GRID_SIZE: i32 = 25;

impl EditorMode {
    pub fn align(&self, position: Vector2<i32>) -> Vector2<i32> {
        match self {
            Self::Grid => position / GRID_SIZE.into() * GRID_SIZE.into(),
            _ => position,
        }
    }
}

// defines at which point the grid doesn't get rendered anymore
const SCALE_CUTOFF: f64 = 0.30;

impl Renderable for EditorMode {
    fn render<R>(&self, renderer: &R, _data: &crate::simulator::Plot) -> Result<(), R::Error>
    where
        R: crate::renderer::Renderer,
    {
        match self {
            EditorMode::Grid => {
                if renderer.scale() < SCALE_CUTOFF {
                    return Ok(());
                }

                let Vector2(start, end) = renderer.screen_space();
                let offset = VectorCast::cast(start) / GRID_SIZE.into() * GRID_SIZE.into();

                renderer.set_color(unsafe { &COLOR_THEME.grid_color });
                for i in (offset.0..end.0 as i32).step_by(GRID_SIZE as usize) {
                    for j in (offset.1..end.1 as i32).step_by(GRID_SIZE as usize) {
                        renderer
                            .rectangle(Vector2(i - 1, j - 1), Vector2(2, 2))
                            .fill()?;
                    }
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }
}
