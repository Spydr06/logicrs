use crate::{simulator::Plot, selection::*};

use super::*;
use gtk::cairo::{
    Context,
    Antialias,
    Error, FontFace
};

pub struct CairoRenderer {
    size: (i32, i32),
    scale: f64,
    translation: (f64, f64),
    original_translation: (f64, f64),
    font: FontFace,
    context: Option<Context>
}

impl CairoRenderer {
    pub fn new() -> Self {
        Self {
            size: (0, 0),
            scale: DEFAULT_SCALE,
            translation: (0., 0.),
            original_translation: (0., 0.),
            context: None,
            font: FontFace::toy_create("Cascadia Code", gtk::cairo::FontSlant::Normal, gtk::cairo::FontWeight::Normal).unwrap()
        }
    }

    #[inline]
    fn set_context(&mut self, context: Option<Context>) -> &mut Self {
        self.context = context;
        self
    }
}

impl CairoRenderer {
    pub fn original_translation(&self) -> (f64, f64) {
        self.original_translation
    }

    pub fn save_translation(&mut self) -> &mut Self {
        self.original_translation = self.translation;
        self
    }
}

impl Default for CairoRenderer {
    fn default() -> Self { Self::new() }
}

impl Renderer for CairoRenderer {
    type Context = cairo::Context;
    type Error = cairo::Error;

    fn callback(&mut self, plot: &Plot, mode: EditorMode, _area: &DrawingArea, context: &Self::Context, width: i32, height: i32) -> Result<&mut Self, Self::Error> {
        self.set_size((width, height)).set_context(Some(context.clone()));     
        if width == 0 || height == 0 {
            return Ok(self);
        }

        let screen_center = (width as f64 / 2., height as f64 / 2.);

        context.translate(screen_center.0, screen_center.1);
        context.scale(self.scale, self.scale);
        context.translate(self.translation.0 - screen_center.0, self.translation.1 - screen_center.1);

        context.set_antialias(Antialias::Default);
        context.set_source_rgb(0.1, 0.1, 0.1);
        context.paint()?;

        context.set_font_face(&self.font);
        context.set_font_size(DEFAULT_FONT_SIZE);

        mode.render(self, plot)?;
        plot.render(self, plot)?;

        // render selection
        plot.selection().render(self, plot).map(|_| self)
    }

    #[inline]
    fn size(&self) -> (i32, i32) {
        self.size
    }

    #[inline]
    fn set_size(&mut self, size: (i32, i32)) -> &mut Self {
        self.size = size;
        self
    }

    #[inline]
    fn translate(&mut self, translation: (f64, f64)) -> &mut Self {
        self.translation = translation;
        self
    }

    #[inline]
    fn translation(&self) -> (f64, f64) {
        self.translation
    }

    #[inline]
    fn scale(&self) -> f64 {
        self.scale
    }

    #[inline]
    fn set_scale(&mut self, scale: f64) -> &mut Self {
        self.scale = scale.clamp(MINIMUM_SCALE, MAXIMUM_SCALE);
        self
    }

    #[inline]
    fn zoom(&mut self, amount: f64) -> &mut Self {
        self.set_scale(self.scale * amount);
        self
    }

    #[inline]
    fn set_color(&self, color: &Color) -> &Self {
        if let Some(context) = &self.context {
            context.set_source_rgba(color.0, color.1, color.2, color.3);
        }
        self
    }

    #[inline]
    fn set_line_width(&self, width: f64) -> &Self {
        if let Some(context) = &self.context {
            context.set_line_width(width);
        }
        self
    }

    #[inline]
    fn set_font_size(&self, size: f64) -> &Self {
        if let Some(context) = &self.context {
            context.set_font_size(size);
        }
        self
    }

    #[inline]
    fn fill(&self) -> Result<&Self, Self::Error> {
        match &self.context {
            Some(context) => context.fill().map(|_| self),
            None => Ok(self) // TODO: error handling
        }
    }

    #[inline]
    fn fill_preserve(&self) -> Result<&Self, Self::Error> {
        match &self.context {
            Some(context) => context.fill_preserve().map(|_| self),
            None => Ok(self) // TODO: error handling
        }
    }

    #[inline]
    fn stroke(&self) -> Result<&Self, Self::Error> {
        match &self.context {
            Some(context) => context.stroke().map(|_| self),
            None => Ok(self) // TODO: error handling
        }
    }

    #[inline]
    fn show_text<'a>(&self, text: &'a str) -> Result<&Self, Error> {
        match &self.context {
            Some(context) => context.show_text(text).map(|_| self),
            None => Ok(self)
        }
    }

    #[inline]
    fn arc(&self, position: (i32, i32), radius: f64, angle1: f64, angle2: f64) -> &Self {
        if let Some(context) = &self.context {
            context.arc(position.0 as f64, position.1 as f64, radius, angle1, angle2);
        }
        self
    }

    #[inline]
    fn rectangle(&self, position: (i32, i32), size: (i32, i32)) -> &Self {
        if let Some(context) = &self.context {
            context.rectangle(position.0 as f64, position.1 as f64, size.0 as f64, size.1 as f64);
        }
        self
    }

    #[inline]
    fn move_to(&self, position: (i32, i32)) -> &Self {
        if let Some(context) = &self.context {
            context.move_to(position.0 as f64, position.1 as f64);
        }
        self
    }

    #[inline]
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

    #[inline]
    fn line_to(&self, position: (i32, i32)) -> &Self {
        if let Some(context) = &self.context {
            context.line_to(position.0 as f64, position.1 as f64);
        }
        self
    }
}