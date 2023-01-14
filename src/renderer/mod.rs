use gtk::DrawingArea;

pub mod cairo;
pub mod color;

pub use {cairo::*, color::*};
use crate::simulator::Plot;

pub const DEFAULT_SCALE: f64 = 1.;
pub const MINIMUM_SCALE: f64 = 0.1;
pub const MAXIMUM_SCALE: f64 = 2.;
pub const DEFAULT_FONT_SIZE: f64 = 15.0;

pub trait Renderable {
    fn render<R>(&self, renderer: &R, data: &Plot) -> Result<(), R::Error>
        where R: Renderer;
}

pub trait Renderer: Default {
    type Context;
    type Error;

    // render callback
    fn callback(&mut self, data: &Plot, _area: &DrawingArea, context: &Self::Context, width: i32, height: i32) -> Result<(), Self::Error>;

    // getter/setter
    fn translate(&mut self, translation: (f64, f64)) -> &mut Self;
    fn translation(&self) -> (f64, f64);
    fn size(&self) -> (i32, i32);
    fn set_size(&mut self, size: (i32, i32)) -> &mut Self;
    fn scale(&self) -> f64;
    fn set_scale(&mut self, scale: f64) -> &mut Self;
    fn zoom(&mut self, amount: f64) -> &mut Self;
    fn set_color(&self, color: &Color) -> &Self;
    fn set_line_width(&self, width: f64) -> &Self;
    fn set_font_size(&self, size: f64) -> &Self;

    fn world_coords(&self, x: f64, y: f64) -> (i32, i32) {
        let screen_center = (self.size().0 / 2, self.size().1 / 2);
        (
            ((x - screen_center.0 as f64) / self.scale() - self.translation().0) as i32 + screen_center.0,
            ((y - screen_center.1 as f64) / self.scale() - self.translation().1) as i32 + screen_center.1
        )
    }

    fn screen_space(&self) -> (i32, i32, i32, i32) {
        let ((a, b), (c, d)) = (
            self.world_coords(0., 0.),
            self.world_coords(self.size().0 as f64, self.size().1 as f64)
        );
        (a, b, c, d)
    }

    // shape functions
    fn arc(&self, position: (i32, i32), radius: f64, angle1: f64, angle2: f64) -> &Self;
    fn rectangle(&self, position: (i32, i32), size: (i32, i32)) -> &Self;

    fn move_to(&self, position: (i32, i32)) -> &Self;
    fn curve_to(&self, start: (i32, i32), mid: (i32, i32), end: (i32, i32)) -> &Self;
    fn line_to(&self, position: (i32, i32)) -> &Self;

    // drawing functions
    fn fill(&self) -> Result<(), Self::Error>;
    fn fill_preserve(&self) -> Result<(), Self::Error>;
    fn stroke(&self) -> Result<(), Self::Error>;
    fn show_text<'a>(&self, text: &'a str) -> Result<(), Self::Error>;

    //
    // more complex shapes building on the backend-specific basic functions
    //

    fn rounded_rect(&self, position: (i32, i32), size: (i32, i32), radius: i32) -> &Self {
        self.move_to((position.0 + radius, position.1));

        self.line_to((position.0 + size.0 - radius, position.1));
        self.curve_to(
            (position.0 + size.0 - radius, position.1), 
            (position.0 + size.0, position.1), 
            (position.0 + size.0, position.1 + radius), 
        );
    
        self.line_to((position.0 + size.0, position.1 + size.1 - radius));
        self.curve_to(
            (position.0 + size.0, position.1 + size.1 - radius),
            (position.0 + size.0, position.1 + size.1),
            (position.0 + size.0 - radius, position.1 + size.1),
        );
    
        self.line_to((position.0 + radius, position.1 + size.1));
        self.curve_to(
            (position.0 + radius, position.1 + size.1),
            (position.0, position.1 + size.1),
            (position.0, position.1 + size.1 - radius)
        );
    
        self.line_to((position.0, position.1 + radius));
        self.curve_to(
            (position.0, position.1 + radius),
            position,
            (position.0 + radius, position.1)
        )
    }

    fn top_rounded_rect(&self, position: (i32, i32), size: (i32, i32), radius: i32) -> &Self {
        self.move_to((position.0 + radius, position.1));
        self.line_to((position.0 + size.0 - radius, position.1));
        self.curve_to(
            (position.0 + size.0 - radius, position.1), 
            (position.0 + size.0, position.1), 
            (position.0 + size.0, position.1 + radius), 
        );
    
        self.line_to((position.0 + size.0, position.1 + size.1));
        self.line_to((position.0, position.1 + size.1));
        self.line_to((position.0, position.1 + radius));
        self.curve_to(
            (position.0, position.1 + radius),
            position,
            (position.0 + radius, position.1)
        )
    }
}