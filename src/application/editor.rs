use crate::renderer::{Renderable, COLOR_THEME};

#[derive(Default, Copy, Clone)]
pub enum EditorMode {
    #[default]
    Normal,
    Grid
}

impl From<bool> for EditorMode {
    fn from(b: bool) -> Self {
        match b {
            true  => Self::Grid,
            false => Self::Normal
        }
    }
}

// grid size in pixels at scale 1.0
pub const GRID_SIZE: i32 = 25;

impl Renderable for EditorMode {
    fn render<R>(&self, renderer: &R, _data: &crate::simulator::Plot) -> Result<(), R::Error>
        where R: crate::renderer::Renderer {
        match self {
            EditorMode::Grid => {
                let (start, end) = renderer.screen_space();
                let offset = (
                    start.0 / GRID_SIZE * GRID_SIZE,
                    start.1 / GRID_SIZE * GRID_SIZE
                );

                renderer.set_color(unsafe { &COLOR_THEME.grid_color });
                for i in (offset.0..end.0).step_by(GRID_SIZE as usize) {
                    for j in (offset.1..end.1).step_by(GRID_SIZE as usize) {
                        renderer.rectangle((i - 1, j - 1), (2, 2)).fill()?;
                    }
                }

                Ok(())
            }
            _ => Ok(())
        }
    }
}
