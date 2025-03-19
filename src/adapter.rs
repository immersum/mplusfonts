use embedded_graphics::Pixel;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Dimensions;
use embedded_graphics::pixelcolor::{BinaryColor, Gray2, Gray4, Gray8};
use embedded_graphics::primitives::Rectangle;

use crate::color::Colormap;

/// Adapter draw target using a colormap.
///
/// This draw target uses a lookup table to get colors of the type that is expected by another draw
/// target.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ColormapAdapter<'a, D: DrawTarget, const N: usize> {
    parent: &'a mut D,
    colormap: &'a Colormap<D::Color, N>,
}

/// Extension trait for draw targets.
pub trait DrawTargetExt: DrawTarget + Sized {
    /// Returns an adapter for the draw target with the specified colormap, for mapping gray values.
    fn value_mapped<'a, const N: usize>(
        &'a mut self,
        colormap: &'a Colormap<Self::Color, N>,
    ) -> ColormapAdapter<'a, Self, N>;
}

impl<D: DrawTarget> DrawTargetExt for D {
    fn value_mapped<'a, const N: usize>(
        &'a mut self,
        colormap: &'a Colormap<D::Color, N>,
    ) -> ColormapAdapter<'a, D, N> {
        ColormapAdapter::new(self, colormap)
    }
}

impl<'a, D: DrawTarget, const N: usize> ColormapAdapter<'a, D, N> {
    /// Creates a new adapter draw target with the specified parent draw target and colormap.
    pub const fn new(parent: &'a mut D, colormap: &'a Colormap<D::Color, N>) -> Self {
        Self { parent, colormap }
    }
}

impl<D: DrawTarget, const N: usize> Dimensions for ColormapAdapter<'_, D, N> {
    fn bounding_box(&self) -> Rectangle {
        self.parent.bounding_box()
    }
}

macro_rules! impl_draw_target {
    (
        $(
            $color_type:ty, $array_length:literal,
        )*
    ) => {
        $(
            impl<D: DrawTarget> DrawTarget for ColormapAdapter<'_, D, $array_length>
            {
                type Color = $color_type;
                type Error = D::Error;

                fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
                where
                    I: IntoIterator<Item = Pixel<Self::Color>>
                {
                    let pixels = pixels.into_iter().map(|Pixel(pixel, color)| {
                        Pixel(pixel, self.colormap.get(color))
                    });

                    self.parent.draw_iter(pixels)
                }

                fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I)-> Result<(), Self::Error>
                where
                    I: IntoIterator<Item = Self::Color>
                {
                    let colors = colors.into_iter().map(|color| self.colormap.get(color));

                    self.parent.fill_contiguous(area, colors)
                }

                fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
                    self.parent.fill_solid(area, self.colormap.get(color))
                }

                fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
                    self.parent.clear(self.colormap.get(color))
                }
            }
        )*
    }
}

impl_draw_target! {
    BinaryColor, 2,
    Gray2, 4,
    Gray4, 16,
    Gray8, 256,
}
