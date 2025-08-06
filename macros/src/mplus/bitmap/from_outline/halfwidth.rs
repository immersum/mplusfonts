#[derive(Clone, Copy)]
pub struct Halfwidth(pub f32);

#[derive(Clone, Copy)]
pub enum PixelAlignmentStrategy {
    Floor(Halfwidth),
    Ceil,
    Zero,
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
}
