use crate::mplus::Font;
use crate::mplus::bitmap::units::Halfwidth;

pub struct GlyphSpacing {
    pub halfwidth: Halfwidth,
    pub is_code: bool,
}

impl GlyphSpacing {
    pub fn from_font(font: &Font, pixels_per_em: f32) -> Self {
        let halfwidth = Halfwidth::from_font(font, pixels_per_em);
        let is_code = matches!(font, Font::MPLUSCode { .. });

        Self { halfwidth, is_code }
    }

    pub fn halfwidths(&self, advance_width: f32) -> f32 {
        if self.is_code && advance_width.abs() > 0.0 {
            match self.halfwidth {
                Halfwidth::Floor(halfwidth) => {
                    let absolute_value = advance_width.signum() * advance_width.floor();
                    let mut new_absolute_value = halfwidth.floor();
                    while absolute_value > new_absolute_value + halfwidth * 0.4 {
                        new_absolute_value += halfwidth;
                    }

                    advance_width.signum() * new_absolute_value.floor()
                }
                Halfwidth::Ceil => advance_width.signum() * (advance_width.abs() / 1.2).ceil(),
                Halfwidth::Zero => advance_width.signum() * 0.0,
            }
        } else {
            advance_width
        }
    }

    pub fn compensate(&self, x_offset: f32) -> f32 {
        if self.is_code && x_offset.abs() > 0.0 {
            match self.halfwidth {
                Halfwidth::Floor(halfwidth) => x_offset - x_offset.signum() * halfwidth.fract(),
                Halfwidth::Ceil => x_offset.signum(),
                Halfwidth::Zero => x_offset.signum() * 0.0,
            }
        } else {
            x_offset
        }
    }
}

mod tests {
    macro_rules! test_halfwidths {
        (
            $(
                $fn_ident:ident, $halfwidth:expr, $is_code:expr, $advance_width:expr, $expected:expr,
            )*
        ) => {
            $(
                #[test]
                fn $fn_ident() {
                    let halfwidth = $halfwidth;
                    let is_code = $is_code;
                    let result = super::GlyphSpacing { halfwidth, is_code }.halfwidths($advance_width);
                    assert_eq!(result, $expected);
                    assert_eq!(result.signum(), f32::signum($expected));
                }
            )*
        }
    }

    test_halfwidths! {
        halfwidths_from_15_to_2_times_9, super::Halfwidth::Floor(9.0), true, 15.0, 2.0 * 9.0,
        halfwidths_from_12_point_5_to_9, super::Halfwidth::Floor(9.0), true, 12.5, 9.0,
        halfwidths_from_9_to_9, super::Halfwidth::Floor(9.0), true, 9.0, 9.0,
        halfwidths_from_7_point_5_to_9, super::Halfwidth::Floor(9.0), true, 7.5, 9.0,
        halfwidths_from_plus_1_point_5, super::Halfwidth::Floor(9.0), true, 1.5, 9.0,
        halfwidths_from_minus_1_point_5, super::Halfwidth::Floor(9.0), true, -1.5, -9.0,

        halfwidths_from_1_point_25, super::Halfwidth::Ceil, true, 1.25, 2.0,
        halfwidths_from_minus_1, super::Halfwidth::Ceil, true, -1.0, -1.0,
        halfwidths_from_plus_0, super::Halfwidth::Ceil, true, 0.0, 0.0,
        halfwidths_from_minus_0, super::Halfwidth::Ceil, true, -0.0, -0.0,

        halfwidths_from_plus_0_point_5, super::Halfwidth::Zero, true, 0.5, 0.0,
        halfwidths_from_minus_0_point_5, super::Halfwidth::Zero, true, -0.5, -0.0,
    }
}
