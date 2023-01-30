use crate::renderer::{Renderable, DEFAULT_THEME};

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
                let start = (
                    -(renderer.translation().0 + renderer.size().0 as f64 * 0.5 / renderer.scale()) as i32,
                    -(renderer.translation().1 + renderer.size().1 as f64 * 0.5 / renderer.scale()) as i32
                );
                let end = (
                    start.0 + ((renderer.size().0) as f64 * 1.5 / renderer.scale()) as i32,
                    start.1 + ((renderer.size().1) as f64 * 1.5 / renderer.scale()) as i32
                );

                renderer.set_color(&DEFAULT_THEME.grid_color)
                    .set_line_width(1.);

                (start.0 as i32 / GRID_SIZE * GRID_SIZE .. end.0).step_by(GRID_SIZE as usize).for_each(|i| {
                    renderer.move_to((i, start.1 as i32)).line_to((i, end.1));
                });

                (start.1 as i32 / GRID_SIZE * GRID_SIZE .. end.1).step_by(GRID_SIZE as usize).for_each(|i| {
                    renderer.move_to((start.0 as i32, i)).line_to((end.0, i));
                });
                
                renderer.stroke().map(|_| ())
            }
            _ => Ok(())
        }
    }
}
