pub type Color = (f64, f64, f64, f64);
pub type Hex = u32;

pub const fn hex_to_color(hex: Hex) -> (f64, f64, f64, f64) {
    (
        ((hex >> 16) & 0xff) as f64 / 255.0,
        ((hex >> 8) & 0xff)  as f64 / 255.0,
        (hex & 0xff)         as f64 / 255.0,
        ((hex >> 24) & 0xff) as f64 / 255.0
    )
}

pub struct Theme {
    // base colors
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

    // decoration color
    pub decoration_fg_color: Color,
}

pub const DEFAULT_THEME: Theme = Theme {
    border_color: (0.23, 0.23, 0.23, 1.),
    block_bg_color: (0.13, 0.13, 0.13, 1.),
    block_fg_color: hex_to_color(0xffffffff),

    grid_color: (0.14, 0.14, 0.14, 1.),

    accent_bg_color: hex_to_color(0x403584e4),
    accent_fg_color: hex_to_color(0xff3584e4),

    disabled_bg_color: hex_to_color(0x809141ac),
    disabled_fg_color: hex_to_color(0xff9141ac),
    enabled_bg_color: hex_to_color(0xff26a269),
    enabled_fg_color: hex_to_color(0xff33d17a),
    suggestion_fg_color: hex_to_color(0xfff9f06b),

    decoration_fg_color: (0.8, 0.8, 0.8, 1.0),
};
