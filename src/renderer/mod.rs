use gtk::DrawingArea;

pub mod cairo;
pub mod color;
pub mod vector;

pub use {cairo::*, color::*};
use crate::{simulator::Plot, application::editor::EditorMode};

use self::vector::Vector2;

pub const DEFAULT_SCALE: f64 = 1.;
pub const MINIMUM_SCALE: f64 = 0.1;
pub const MAXIMUM_SCALE: f64 = 2.;
pub const DEFAULT_FONT_SIZE: f64 = 15.0;

pub type ScreenSpace = Vector2<Vector2<f64>>;

pub trait Renderable {
    fn render<R>(&self, renderer: &R, data: &Plot) -> Result<(), R::Error>
        where R: Renderer;
}

pub trait Renderer: Default {
    type Context;
    type Error;

    // render callback
    fn callback(&mut self, data: &Plot, mode: EditorMode, area: &DrawingArea, context: &Self::Context, width: i32, height: i32) -> Result<&mut Self, Self::Error>;

    // getter/setter
    fn translate(&mut self, translation: Vector2<f64>) -> &mut Self;
    fn translation(&self) -> Vector2<f64>;
    fn size(&self) -> Vector2<i32>;
    fn set_size(&mut self, size: Vector2<i32>) -> &mut Self;
    fn scale(&self) -> f64;
    fn set_scale(&mut self, scale: f64) -> &mut Self;
    fn set_color(&self, color: &Color) -> &Self;
    fn set_line_width(&self, width: f64) -> &Self;
    fn set_font_size(&self, size: f64) -> &Self;
    fn set_editor_mode(&mut self, mode: EditorMode);
    fn editor_mode(&self) -> EditorMode;

    fn screen_space(&self) -> ScreenSpace {
        Vector2(
            self.screen_to_world(Vector2::default()),
            self.screen_to_world(Vector2(self.size().0 as f64, self.size().1 as f64))
        )
    }

    fn screen_to_world(&self, position: Vector2<f64>) -> Vector2<f64> {
        (position - self.translation()) / self.scale().into()
    }

    fn world_to_screen(&self, position: Vector2<f64>) -> Vector2<f64> {
        position * self.scale().into() + self.translation()
    }

    fn zoom(&mut self, amount: f64, screen_position: Option<Vector2<f64>>) {
        let screen_position = match screen_position {
            Some(position) => position,
            None => (self.size().0 as f64 / 2., self.size().1 as f64 / 2.).into()
        };

        let p = self.screen_to_world(screen_position);
        self.set_scale(self.scale() * amount);
        let n = self.world_to_screen(p);
        self.translate(self.translation() - n + screen_position);
    }

    // shape functions
    fn arc(&self, position: Vector2<i32>, radius: f64, angle1: f64, angle2: f64) -> &Self;
    fn rectangle(&self, position: Vector2<i32>, size: Vector2<i32>) -> &Self;

    fn move_to(&self, position: Vector2<i32>) -> &Self;
    fn curve_to(&self, start: Vector2<i32>, mid: Vector2<i32>, end: Vector2<i32>) -> &Self;
    fn line_to(&self, position: Vector2<i32>) -> &Self;

    // drawing functions
    fn fill(&self) -> Result<&Self, Self::Error>;
    fn fill_preserve(&self) -> Result<&Self, Self::Error>;
    fn stroke(&self) -> Result<&Self, Self::Error>;
    fn show_text<'a>(&self, text: &'a str) -> Result<&Self, Self::Error>;

    //
    // more complex shapes building on the backend-specific basic functions
    //

    fn rounded_rect(&self, position: Vector2<i32>, size: Vector2<i32>, radius: i32) -> &Self {
        self.move_to(position + Vector2(radius, 0));

        self.line_to(position + Vector2(size.0 - radius, 0));
        self.curve_to(
            Vector2(position.0 + size.0 - radius, position.1), 
            Vector2(position.0 + size.0, position.1), 
            Vector2(position.0 + size.0, position.1 + radius), 
        );
    
        self.line_to(Vector2(position.0 + size.0, position.1 + size.1 - radius));
        self.curve_to(
            Vector2(position.0 + size.0, position.1 + size.1 - radius),
            Vector2(position.0 + size.0, position.1 + size.1),
            Vector2(position.0 + size.0 - radius, position.1 + size.1),
        );
    
        self.line_to(Vector2(position.0 + radius, position.1 + size.1));
        self.curve_to(
            Vector2(position.0 + radius, position.1 + size.1),
            Vector2 (position.0, position.1 + size.1),
            Vector2(position.0, position.1 + size.1 - radius)
        );
    
        self.line_to(Vector2(position.0, position.1 + radius));
        self.curve_to(
            Vector2(position.0, position.1 + radius),
            position,
            Vector2(position.0 + radius, position.1)
        )
    }

    fn top_rounded_rect(&self, position: Vector2<i32>, size: Vector2<i32>, radius: i32) -> &Self {
        self.move_to(Vector2(position.0 + radius, position.1));
        self.line_to(Vector2(position.0 + size.0 - radius, position.1));
        self.curve_to(
            Vector2(position.0 + size.0 - radius, position.1), 
            Vector2(position.0 + size.0, position.1), 
            Vector2(position.0 + size.0, position.1 + radius), 
        );
    
        self.line_to(Vector2(position.0 + size.0, position.1 + size.1));
        self.line_to(Vector2(position.0, position.1 + size.1));
        self.line_to(Vector2(position.0, position.1 + radius));
        self.curve_to(
            Vector2(position.0, position.1 + radius),
            position,
            Vector2(position.0 + radius, position.1)
        )
    }
}
