use std::cmp;

use gtk::{
    DrawingArea,
    cairo::{
        Context,
        Antialias,
        Error, FontFace
    }
};

pub mod cairo;
pub use cairo::*;

pub trait Renderable {
    fn render(&self, renderer: &impl Renderer) -> Result<(), Error>;
}

pub trait Renderer {
    // getter/setter
    fn size(&self) -> (i32, i32);
    fn set_size(&mut self, size: (i32, i32)) -> &mut Self;
    fn scale(&self) -> f64;
    fn set_scale(&mut self, scale: f64) -> &mut Self;
    fn set_color(&self, red: f64, green: f64, blue: f64, alpha: f64) -> &Self;

    // shape functions
    fn rounded_rect(&self, position: (i32, i32), size: (i32, i32), radius: i32) -> &Self;
    fn top_rounded_rect(&self, position: (i32, i32), size: (i32, i32), radius: i32) -> &Self;
    fn arc(&self, position: (i32, i32), radius: f64, angle1: f64, angle2: f64) -> &Self;
    fn move_to(&self, position: (i32, i32)) -> &Self;

    // drawing functions
    fn fill(&self) -> Result<(), Error>;
    fn stroke(&self) -> Result<(), Error>;
    fn show_text<'a>(&self, text: &'a str) -> Result<(), Error>;
}