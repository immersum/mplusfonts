use crate::mplus::Font;
use crate::mplus::bitmap::units::Halfwidth;

pub struct GlyphAligner {
    pub is_code: bool,
    pub dir: GlyphAlignDir,
}

pub enum GlyphAlignDir {
    Floor(Halfwidth),
    Ceil,
    Unit,
}

impl GlyphAligner {
    pub fn from_font(font: &Font, pixels_per_em: f32) -> Self {
        let is_code = matches!(font, Font::MPLUSCode { .. });
        let dir = GlyphAlignDir::from_font(font, pixels_per_em);

        Self { is_code, dir }
    }

    pub fn round_halfwidths(&self, advance_width: f32) -> f32 {
        if let Self { is_code: true, dir } = self {
            dir.halfwidths(advance_width)
        } else {
            advance_width
        }
    }

    pub fn round_counteract(&self, x_offset: f32) -> f32 {
        if let Self { is_code: true, dir } = self {
            dir.counteract(x_offset)
        } else {
            x_offset
        }
    }
}

impl GlyphAlignDir {
    pub fn new(pixels_per_em: f32, halfwidth: Halfwidth) -> Self {
        match pixels_per_em {
            ..1.25 => Self::Unit,
            ..2.0 => Self::Ceil,
            _ => Self::Floor(halfwidth),
        }
    }

    pub fn from_font(font: &Font, pixels_per_em: f32) -> Self {
        let halfwidth = Halfwidth::from_font(font, pixels_per_em);

        Self::new(pixels_per_em, halfwidth)
    }

    pub fn halfwidths(&self, advance_width: f32) -> f32 {
        if advance_width.abs() > 0.0 {
            match *self {
                Self::Floor(Halfwidth(halfwidth)) => {
                    let advance_width = advance_width.signum() * advance_width.floor();
                    let mut new_advance_width = halfwidth.floor();
                    while advance_width > new_advance_width + halfwidth * 0.4 {
                        new_advance_width += halfwidth;
                    }

                    advance_width.signum() * new_advance_width.floor()
                }
                Self::Ceil => (advance_width / 1.2).ceil(),
                Self::Unit => 1.0,
            }
        } else {
            advance_width
        }
    }

    pub fn counteract(&self, x_offset: f32) -> f32 {
        if x_offset < 0.0 {
            if let Self::Floor(Halfwidth(halfwidth)) = *self {
                x_offset + halfwidth % 1.0
            } else {
                x_offset
            }
        } else {
            x_offset
        }
    }
}
