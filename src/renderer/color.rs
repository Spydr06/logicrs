pub type Color = (f32, f32, f32, f32);
pub type Hex = u32;

pub trait IntoColor {
    fn into_color(self) -> Color;
}

impl IntoColor for gtk::gdk::RGBA {
    fn into_color(self) -> Color {
        (self.red(), self.green(), self.blue(), self.alpha())
    }
}

pub trait IntoRGBA {
    fn into_rgba(self) -> gtk::gdk::RGBA;
}

impl IntoRGBA for Color {
    fn into_rgba(self) -> gtk::gdk::RGBA {
        gtk::gdk::RGBA::new(self.0, self.1, self.2, self.3)
    }
}

pub const fn hex_to_color(hex: Hex) -> (f32, f32, f32, f32) {
    (
        ((hex >> 16) & 0xff) as f32 / 255.0,
        ((hex >> 8) & 0xff) as f32 / 255.0,
        (hex & 0xff) as f32 / 255.0,
        ((hex >> 24) & 0xff) as f32 / 255.0,
    )
}

pub static mut COLOR_THEME: Theme = Theme::DARK;

#[derive(Copy, Clone)]
pub struct Theme {
    // base colors
    pub bg_color: Color,
    pub border_color: Color,
    pub block_bg_color: Color,
    pub block_fg_color: Color,

    pub grid_color: Color,

    // accent colors (selection, etc.)
    pub accent_bg_color: Color,
    pub accent_fg_color: Color,

    // wire and connector colors
    pub disabled_bg_color: Color,
    pub disabled_fg_color: Color,
    pub enabled_bg_color: Color,
    pub enabled_fg_color: Color,
    pub suggestion_fg_color: Color,

    // button colors
    pub button_active_color: Color,
    pub button_inactive_color: Color,

    // decoration color
    pub decoration_fg_color: Color,
}

impl From<&adw::StyleManager> for Theme {
    fn from(style_manager: &adw::StyleManager) -> Self {
        if style_manager.is_dark() {
            Self::DARK
        } else {
            Self::LIGHT
        }
    }
}

impl Theme {
    pub fn init() {
        let style_manager = adw::StyleManager::default();
        style_manager.connect_dark_notify(|style_manager| unsafe {
            COLOR_THEME = Self::from(style_manager)
        });
        unsafe { COLOR_THEME = Self::from(&style_manager) }
    }

    const DARK: Self = Self {
        bg_color: (0.1, 0.1, 0.1, 1.),
        border_color: (0.23, 0.23, 0.23, 1.),
        block_bg_color: (0.13, 0.13, 0.13, 1.),
        block_fg_color: hex_to_color(0xffffffff),

        grid_color: (0.23, 0.23, 0.23, 1.),

        accent_bg_color: hex_to_color(0x403584e4),
        accent_fg_color: hex_to_color(0xff3584e4),

        disabled_bg_color: hex_to_color(0x809141ac),
        disabled_fg_color: hex_to_color(0xff9141ac),
        enabled_bg_color: hex_to_color(0xff26a269),
        enabled_fg_color: hex_to_color(0xff33d17a),
        suggestion_fg_color: hex_to_color(0xfff9f06b),

        button_active_color: hex_to_color(0xffed333b),
        button_inactive_color: hex_to_color(0xaaa51d2d),

        decoration_fg_color: (0.8, 0.8, 0.8, 1.0),
    };

    const LIGHT: Self = Self {
        bg_color: hex_to_color(0xfffafafa),
        border_color: (0.65, 0.65, 0.65, 1.),
        block_bg_color: hex_to_color(0xfffafafa),
        block_fg_color: hex_to_color(0xff000000),

        grid_color: (0.23, 0.23, 0.23, 1.),

        accent_bg_color: hex_to_color(0x401c71d8),
        accent_fg_color: hex_to_color(0xff1c71d8),

        disabled_bg_color: hex_to_color(0x809141ac),
        disabled_fg_color: hex_to_color(0xff9141ac),
        enabled_bg_color: hex_to_color(0xff26a269),
        enabled_fg_color: hex_to_color(0xff33d17a),
        suggestion_fg_color: hex_to_color(0xfff9f06b),

        button_active_color: hex_to_color(0xffed333b),
        button_inactive_color: hex_to_color(0xaaa51d2d),

        decoration_fg_color: (0.0, 0.0, 0.0, 1.0),
    };
}
