use super::*;

pub const DEFAULT_SCALE: f64 = 1.;
pub const MINIMUM_SCALE: f64 = 0.1;
pub const MAXIMUM_SCALE: f64 = 2.;

pub struct CairoRenderer {
    size: (i32, i32),
    scale: f64,
    font: FontFace,
    context: Option<Context>
}

impl CairoRenderer {
    pub fn new() -> Self {
        Self {
            size: (0, 0),
            scale: DEFAULT_SCALE,
            context: None,
            font: FontFace::toy_create("Cascadia Code", gtk::cairo::FontSlant::Normal, gtk::cairo::FontWeight::Normal).unwrap()
        }
    }

    fn set_context(&mut self, context: Option<Context>) -> &mut Self {
        self.context = context;
        self
    }

    pub fn render_callback(&mut self, _area: &DrawingArea, context: &Context, width: i32, height: i32) -> Result<(), Error> {
        self.set_size((width, height)).set_context(Some(context.clone()));     
        if width == 0 || height == 0 {
            return Ok(());
        }

        context.scale(self.scale, self.scale);

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
                if block.is_in_area((0, 0, (width as f64 / self.scale) as i32, (height as f64 / self.scale) as i32)) {
                    block.render(self)?;
                }
            }

            // draw selection rectangle
            if let Some((start_x, start_y)) = data.selection().area_start() {
                if let Some((end_x, end_y)) = data.selection().area_end() {
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

impl Renderer for CairoRenderer {
    fn size(&self) -> (i32, i32) {
        self.size
    }

    fn set_size(&mut self, size: (i32, i32)) -> &mut Self {
        self.size = size;
        self
    }

    fn scale(&self) -> f64 {
        self.scale
    }

    fn set_scale(&mut self, scale: f64) -> &mut Self {
        self.scale = scale.clamp(MINIMUM_SCALE, MAXIMUM_SCALE);
        self
    }

    fn set_color(&self, red: f64, green: f64, blue: f64, alpha: f64) -> &Self {
        if let Some(context) = &self.context {
            context.set_source_rgba(red, green, blue, alpha)
        }

        self
    }

    fn fill(&self) -> Result<(), Error> {
        match &self.context {
            Some(context) => context.fill(),
            None => Ok(()) // TODO: error handling
        }
    }

    fn stroke(&self) -> Result<(), Error> {
        match &self.context {
            Some(context) => context.stroke(),
            None => Ok(()) // TODO: error handling
        }
    }

    fn show_text<'a>(&self, text: &'a str) -> Result<(), Error> {
        match &self.context {
            Some(context) => context.show_text(text),
            None => Ok(())
        }
    }

    fn arc(&self, position: (i32, i32), radius: f64, angle1: f64, angle2: f64) -> &Self {
        if let Some(context) = &self.context {
            context.arc(position.0 as f64, position.1 as f64, radius, angle1, angle2);
        }
        
        self
    }

    fn move_to(&self, position: (i32, i32)) -> &Self {
        if let Some(context) = &self.context {
            context.move_to(position.0 as f64, position.1 as f64);
        }
        self
    }

    fn curve_to(&self, start: (i32, i32), mid: (i32, i32), end: (i32, i32)) -> &Self {
        if let Some(context) = &self.context {
            context.curve_to(
                start.0 as f64, start.1 as f64, 
                mid.0 as f64, mid.1 as f64, 
                end.0 as f64, end.1 as f64
            );
        }
        self
    }

    fn line_to(&self, position: (i32, i32)) -> &Self {
        if let Some(context) = &self.context {
            context.line_to(position.0 as f64, position.1 as f64);
        }
        self
    }

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