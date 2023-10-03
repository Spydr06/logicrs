use crate::renderer::{vector::Vector2, *};
use serde::{Deserialize, Serialize};
use std::f64;

use super::Block;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Decoration {
    None,
    Label(String),
    NotLabel(String),
    Button(bool),
    Switch(bool),
    Lamp(bool),
}

impl Default for Decoration {
    fn default() -> Self {
        Self::None
    }
}

impl Decoration {
    pub(super) fn render<R>(&self, renderer: &R, block: &Block) -> Result<(), R::Error>
    where
        R: Renderer,
    {
        match self {
            Self::Label(label) => {
                renderer
                    .set_font_size(26.0)
                    .move_to(Vector2(
                        block.position().0
                            + (block.size().0 / 2 - 7 * label.chars().count() as i32),
                        block.position().1 + (block.size().1 / 2 + 20),
                    ))
                    .set_color(unsafe { &COLOR_THEME.decoration_fg_color })
                    .show_text(label)?
                    .set_font_size(DEFAULT_FONT_SIZE);
                Ok(())
            }
            Self::NotLabel(label) => {
                let offset = Vector2(
                    7 * label.chars().count() as i32,
                    block.position().1 + block.size().1 / 2 - 2,
                );
                let position =
                    block.position() + (block.size() / 2.into() + Vector2(-offset.0, 20));
                renderer
                    .set_font_size(26.0)
                    .move_to(position)
                    .set_color(unsafe { &COLOR_THEME.decoration_fg_color })
                    .show_text(label)?
                    .set_font_size(DEFAULT_FONT_SIZE)
                    .move_to(Vector2(position.0, offset.1))
                    .set_line_width(2.5)
                    .line_to(Vector2(position.0 + 2 * offset.0, offset.1))
                    .stroke()
                    .map(|_| ())
            }
            Self::Lamp(active) => renderer
                .arc(
                    Vector2(
                        block.position().0 + block.size().0 / 2,
                        block.position().1 + 50,
                    ),
                    12.,
                    0.,
                    f64::consts::TAU,
                )
                .set_color(unsafe {
                    if *active {
                        &COLOR_THEME.suggestion_fg_color
                    } else {
                        &COLOR_THEME.border_color
                    }
                })
                .fill_preserve()?
                .set_line_width(1.5)
                .set_color(unsafe { &COLOR_THEME.border_color })
                .stroke()
                .map(|_| ()),
            Self::Button(active) | Self::Switch(active) => renderer
                .arc(
                    Vector2(
                        block.position().0 + block.size().0 / 2,
                        block.position().1 + 50,
                    ),
                    12.,
                    0.,
                    f64::consts::TAU,
                )
                .set_color(unsafe {
                    if *active {
                        &COLOR_THEME.button_active_color
                    } else {
                        &COLOR_THEME.button_inactive_color
                    }
                })
                .fill_preserve()?
                .set_line_width(1.5)
                .set_color(unsafe { &COLOR_THEME.border_color })
                .stroke()
                .map(|_| ()),
            _ => Ok(()),
        }
    }
}

impl Decoration {
    pub fn set_active(&mut self, is_active: bool) {
        match self {
            Self::Button(active) | Self::Switch(active) | Self::Lamp(active) => *active = is_active,
            _ => {}
        }
    }

    pub fn is_active(&self) -> bool {
        match self {
            Self::Button(active) | Self::Switch(active) | Self::Lamp(active) => *active,
            _ => false,
        }
    }

    pub fn on_mouse_press(&mut self) -> bool {
        match self {
            Self::Switch(active) => {
                *active = !*active;
                true
            }
            Self::Button(active) => {
                *active = true;
                true
            }
            _ => false,
        }
    }

    pub fn on_mouse_release(&mut self) {
        if let Self::Button(active) = self {
            *active = false
        }
    }

    pub fn clickable(&self) -> bool {
        matches!(self, Self::Switch(_) | Self::Button(_))
    }
}
