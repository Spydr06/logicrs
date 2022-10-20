use std::{
    cell::RefCell,
    cmp
};

use gtk::{
    DrawingArea,
    cairo::{
        Context,
        Antialias,
        Error, FontFace
    }
};

pub trait Renderable {
    fn render(&self, context: &Context) -> Result<(), Error>;
}

#[derive(Default)]
struct Data {
    size: (i32, i32)
}

impl Data {
    fn new() -> Self {
        Self {
            size: (0, 0)
        }
    }
}

pub struct Renderer {
    data: RefCell<Data>,
    font: FontFace
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            data: RefCell::new(Data::new()),
            font: FontFace::toy_create("Cascadia Code", gtk::cairo::FontSlant::Normal, gtk::cairo::FontWeight::Normal).unwrap()
        }
    }

    fn set_size(&self, size: (i32, i32)) {
        self.data.borrow_mut().size = size;
    }

    pub fn render_callback(&self, _area: &DrawingArea, context: &Context, width: i32, height: i32) -> Result<(), Error> {
        self.set_size((width, height));       
        if width == 0 || height == 0 {
            return Ok(());
        }

        //println!("renderer::render_callback() called\n  size: ({}, {})", width, height);

        context.set_antialias(Antialias::Default);
        context.set_source_rgb(0.1, 0.1, 0.1);
        context.paint()?;

        context.set_font_face(&self.font);
        context.set_font_size(15.0);

        // render all blocks
        crate::APPLICATION_DATA.with(|d| -> Result<(), Error> {
            let data = d.borrow();
            for block in data.get_blocks() {
                if block.is_in_area((0, 0, width, height)) {
                    block.render(context)?;
                }
            }

            // draw selection rectangle
            if let Some((start_x, start_y)) = data.multiselect_start() {
                if let Some((end_x, end_y)) = data.multiselect_end() {
                    let x = cmp::min(start_x, end_x);
                    let y = cmp::min(start_y, end_y);
                    let w = cmp::max(start_x, end_x) - x;
                    let h = cmp::max(start_y, end_y) - y;

                    context.rectangle(x as f64, y as f64, w as f64, h as f64);
                    context.set_source_rgba(0.2078, 0.5176, 0.894, 0.3);
                    context.fill()?;

                    context.rectangle(x as f64, y as f64, w as f64, h as f64);
                    context.set_source_rgba(0.2078, 0.5176, 0.894, 0.7);
                    context.stroke()?;
                }
            }

            Ok(())
        })
    }
}

pub fn draw_top_rounded_rect(context: &Context, position: (i32, i32), size: (i32, i32), radius: i32) {
    context.move_to((position.0 + radius) as f64, position.1 as f64);
    context.line_to((position.0 + size.0 - radius) as f64, position.1 as f64);
    context.curve_to(
        (position.0 + size.0 - radius) as f64, position.1 as f64, 
        (position.0 + size.0) as f64, position.1 as f64, 
        (position.0 + size.0) as f64, (position.1 + radius) as f64, 
    );

    context.line_to((position.0 + size.0) as f64, (position.1 + size.1) as f64);
    context.line_to(position.0 as f64, (position.1 + size.1) as f64);
    context.line_to(position.0 as f64, (position.1 + radius) as f64);
    context.curve_to(
        position.0 as f64, (position.1 + radius) as f64,
        position.0 as f64, position.1 as f64,
        (position.0 + radius) as f64, position.1 as f64
    );
}

pub fn draw_rounded_rect(context: &Context, position: (i32, i32), size: (i32, i32), radius: i32) {
    context.move_to((position.0 + radius) as f64, position.1 as f64);

    context.line_to((position.0 + size.0 - radius) as f64, position.1 as f64);
    context.curve_to(
        (position.0 + size.0 - radius) as f64, position.1 as f64, 
        (position.0 + size.0) as f64, position.1 as f64, 
        (position.0 + size.0) as f64, (position.1 + radius) as f64, 
    );

    context.line_to((position.0 + size.0) as f64, (position.1 + size.1 - radius) as f64);
    context.curve_to(
        (position.0 + size.0) as f64, (position.1 + size.1 - radius) as f64,
        (position.0 + size.0) as f64, (position.1 + size.1) as f64,
        (position.0 + size.0 - radius) as f64, (position.1 + size.1) as f64,
    );

    context.line_to((position.0 + radius) as f64, (position.1 + size.1) as f64);
    context.curve_to(
        (position.0 + radius) as f64, (position.1 + size.1) as f64,
        position.0 as f64, (position.1 + size.1) as f64,
        position.0 as f64, (position.1 + size.1 - radius) as f64
    );

    context.line_to(position.0 as f64, (position.1 + radius) as f64);
    context.curve_to(
        position.0 as f64, (position.1 + radius) as f64,
        position.0 as f64, position.1 as f64,
        (position.0 + radius) as f64, position.1 as f64
    );
}