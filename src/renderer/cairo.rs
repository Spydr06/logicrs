use crate::{simulator::Plot, application::selection::*};

use super::*;
use gtk::cairo::{
    Context,
    Antialias,
    Error, FontFace
};

pub struct CairoRenderer {
    size: Vector2<i32>,
    scale: f64,
    translation: Vector2<f64>,
    original_translation: Vector2<f64>,
    font: FontFace,
    context: Option<Context>,
    editor_mode: EditorMode
}

impl CairoRenderer {
    pub fn new() -> Self {
        Self {
            size: Vector2::default(),
            scale: DEFAULT_SCALE,
            translation: Vector2::default(),
            original_translation: Vector2::default(),
            context: None,
            editor_mode: EditorMode::default(),
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
    pub fn original_translation(&self) -> Vector2<f64> {
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
        self.set_size(Vector2(width, height)).set_context(Some(context.clone()));     
        self.set_editor_mode(mode);
        if width == 0 || height == 0 {
            return Ok(self);
        }

        // renderer prelude
        context.set_antialias(Antialias::Default);
        context.translate(self.translation.x(), self.translation.y());
        context.scale(self.scale, self.scale);
        
        context.set_font_face(&self.font);
        context.set_font_size(DEFAULT_FONT_SIZE);

        // fill background
        let (bg_color_r, bg_color_g, bg_color_b, _) = unsafe { COLOR_THEME.bg_color };
        context.set_source_rgb(bg_color_r as f64, bg_color_g as f64, bg_color_b as f64);
        context.paint()?;

        // draw the editor grid if enabled
        mode.render(self, plot)?;
        
        // draw the actual contents of the editor
        plot.render(self, plot)?;

        // render selection
        plot.selection().render(self, plot).map(|_| self)
    }

    #[inline]
    fn size(&self) -> Vector2<i32> {
        self.size
    }

    #[inline]
    fn set_size(&mut self, size: Vector2<i32>) -> &mut Self {
        self.size = size;
        self
    }

    #[inline]
    fn translate(&mut self, translation: Vector2<f64>) -> &mut Self {
        self.translation = translation;
        self
    }

    #[inline]
    fn translation(&self) -> Vector2<f64> {
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
    fn set_color(&self, color: &Color) -> &Self {
        if let Some(context) = &self.context {
            context.set_source_rgba(color.0 as f64, color.1 as f64, color.2 as f64, color.3 as f64);
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
    fn arc(&self, position: Vector2<i32>, radius: f64, angle1: f64, angle2: f64) -> &Self {
        if let Some(context) = &self.context {
            context.arc(position.0 as f64, position.1 as f64, radius, angle1, angle2);
        }
        self
    }

    #[inline]
    fn rectangle(&self, position: Vector2<i32>, size: Vector2<i32>) -> &Self {
        if let Some(context) = &self.context {
            context.rectangle(position.0 as f64, position.1 as f64, size.0 as f64, size.1 as f64);
        }
        self
    }

    #[inline]
    fn move_to(&self, position: Vector2<i32>) -> &Self {
        if let Some(context) = &self.context {
            context.move_to(position.0 as f64, position.1 as f64);
        }
        self
    }

    #[inline]
    fn curve_to(&self, start: Vector2<i32>, mid: Vector2<i32>, end: Vector2<i32>) -> &Self {
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
    fn line_to(&self, position: Vector2<i32>) -> &Self {
        if let Some(context) = &self.context {
            context.line_to(position.0 as f64, position.1 as f64);
        }
        self
    }

    #[inline]
    fn set_editor_mode(&mut self, mode: EditorMode) {
        self.editor_mode = mode;
    }

    #[inline]
    fn editor_mode(&self) -> EditorMode {
        self.editor_mode
    }
}
