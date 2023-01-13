use gtk::DrawingArea;

pub mod cairo;
pub use cairo::*;

use crate::simulator::Plot;

pub trait Renderable {
    fn render<R>(&self, renderer: &R, data: &Plot) -> Result<(), R::Error>
        where R: Renderer;
}

pub trait Renderer {
    type Context;
    type Error;

    // render callback
    fn callback(&mut self, data: &Plot, _area: &DrawingArea, context: &Self::Context, width: i32, height: i32) -> Result<(), Self::Error>;

    // getter/setter
    fn size(&self) -> (i32, i32);
    fn set_size(&mut self, size: (i32, i32)) -> &mut Self;
    fn scale(&self) -> f64;
    fn set_scale(&mut self, scale: f64) -> &mut Self;
    fn set_color(&self, red: f64, green: f64, blue: f64, alpha: f64) -> &Self;
    fn set_line_width(&self, width: f64) -> &Self;
    fn set_font_size(&self, size: f64) -> &Self;

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