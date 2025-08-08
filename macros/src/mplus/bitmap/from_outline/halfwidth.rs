use crate::mplus::font::{Font, FontWidth};

#[derive(Clone, Copy)]
pub struct Halfwidth(pub f32);

#[derive(Clone, Copy)]
pub enum PixelAlignmentStrategy {
    Floor(Halfwidth),
    Ceil,
    Zero,
}

#[derive(Clone)]
pub struct AdjustableAdvanceWidth {
    strategy: PixelAlignmentStrategy,
    advance_width: f32,
}

impl Halfwidth {
    pub fn adjustment(&self, advance_width: f32) -> f32 {
        let Self(halfwidth) = *self;
        debug_assert_ne!(0.0, halfwidth);
        debug_assert_eq!(advance_width.signum(), halfwidth.signum());

        let mut new_advance_width = halfwidth.floor();
        while advance_width > new_advance_width + halfwidth * 0.4 {
            new_advance_width += halfwidth;
        }

        new_advance_width = new_advance_width.floor();
        new_advance_width - advance_width
    }
}

impl PixelAlignmentStrategy {
    pub fn new(pixels_per_em: f32, em_per_halfwidth: f32) -> Self {
        match pixels_per_em {
            ..1.25 => PixelAlignmentStrategy::Zero,
            ..2.0 => PixelAlignmentStrategy::Ceil,
            _ => PixelAlignmentStrategy::Floor(Halfwidth(pixels_per_em * em_per_halfwidth)),
        }
    }

    pub fn from_font(font: &Font, pixels_per_em: f32) -> Self {
        let em_per_halfwidth = match *font {
            Font::MPLUSCode {
                variable: (.., FontWidth(units)),
                ..
            } => f32::from(units).mul_add(0.4 / 100.0, 0.1),
            _ => 0.5,
        };

        Self::new(pixels_per_em, em_per_halfwidth)
    }

    pub fn with_advance_width(&self, advance_width: f32) -> AdjustableAdvanceWidth {
        let advance_width = match self {
            PixelAlignmentStrategy::Floor(_) => advance_width.floor(),
            PixelAlignmentStrategy::Ceil => (advance_width / 1.2).ceil(),
            PixelAlignmentStrategy::Zero => 0.0,
        };

        AdjustableAdvanceWidth {
            strategy: *self,
            advance_width,
        }
    }
}

impl AdjustableAdvanceWidth {
    pub fn adjustment(&self) -> f32 {
        match self.strategy {
            PixelAlignmentStrategy::Floor(halfwidth) => halfwidth.adjustment(self.advance_width),
            PixelAlignmentStrategy::Ceil | PixelAlignmentStrategy::Zero => 0.0,
        }
    }

    pub fn into_inner(self) -> f32 {
        self.advance_width
    }
}
