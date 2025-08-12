use crate::mplus::font::{Font, FontWidth};

#[derive(Clone, Copy)]
pub struct Halfwidth(pub f32);

impl Halfwidth {
    pub fn from_font(font: &Font, pixels_per_em: f32) -> Self {
        let em_per_halfwidth = match *font {
            Font::MPLUSCode {
                variable: (.., FontWidth(units)),
                ..
            } => f32::from(units).mul_add(0.4 / 100.0, 0.1),
            _ => 0.5,
        };

        Self(pixels_per_em * em_per_halfwidth)
    }
}
