use embedded_graphics::Drawable;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, OriginDimensions, Size};
use embedded_graphics::image::ImageDrawable;
use embedded_graphics::pixelcolor::{BinaryColor, Gray2, Gray4, Gray8, PixelColor};
use embedded_graphics::primitives::Rectangle;

use crate::color::{Colormap, Invert, Screen};
use crate::image::{Colors, Image, SubImage};

/// Image drawable with references to two overlapping images and a colormap.
///
/// While also performing color conversion, drawing this image drawable involves mixing the colors
/// that form pairs of pixels in [`Screen`] blend mode.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ImageMix<'a, 'b, 'c, U, V, T, const N: usize>
where
    T: PixelColor + Default + Invert + Screen,
    U: ImageDrawable + Colors<U::Color>,
    V: ImageDrawable + Colors<V::Color>,
{
    first: Image<SubImage<'a, U>>,
    second: Image<SubImage<'b, V>>,
    colormap: &'c Colormap<T, N>,
    area: Rectangle,
}

/// Image drawable creation, color-mixed with another image drawable in the area that is their
/// intersection.
pub trait Mixed<U, C, const N: usize>
where
    C: PixelColor + From<C::Raw>,
    U: ImageDrawable<Color = C> + Colors<C>,
{
    /// Returns the image drawable's intersection with the specified image, color-mixed in the
    /// resulting area.
    fn mixed<'b, 'c, V, T>(
        &self,
        other: &'b Image<V>,
        colormap: &'c Colormap<T, N>,
    ) -> ImageMix<'_, 'b, 'c, U, V, T, N>
    where
        V: ImageDrawable<Color = C> + Colors<C>,
        T: PixelColor + Default + Invert + Screen;
}

impl<'a, 'b, 'c, U, V, T, const N: usize> ImageMix<'a, 'b, 'c, U, V, T, N>
where
    T: PixelColor + Default + Invert + Screen,
    U: ImageDrawable + Colors<U::Color>,
    V: ImageDrawable + Colors<V::Color>,
{
    /// Creates a new image drawable with two pre-cut image drawables and the specified colormap.
    const fn new(
        first: Image<SubImage<'a, U>>,
        second: Image<SubImage<'b, V>>,
        colormap: &'c Colormap<T, N>,
        area: Rectangle,
    ) -> Self {
        Self {
            first,
            second,
            colormap,
            area,
        }
    }
}

impl<U, V, T, const N: usize> OriginDimensions for ImageMix<'_, '_, '_, U, V, T, N>
where
    T: PixelColor + Default + Invert + Screen,
    U: ImageDrawable + Colors<U::Color>,
    V: ImageDrawable + Colors<V::Color>,
{
    fn size(&self) -> Size {
        self.area.size
    }
}

macro_rules! impl_drawable {
    (
        $(
            $color_type:ty, $array_length:literal,
        )*
    ) => {
        $(
            impl<U, V, T> Drawable for ImageMix<'_, '_, '_, U, V, T, $array_length>
            where
                T: PixelColor + Default + Invert + Screen,
                U: ImageDrawable<Color = $color_type> + Colors<$color_type>,
                V: ImageDrawable<Color = $color_type> + Colors<$color_type>,
            {
                type Color = T;
                type Output = ();

                fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
                where
                    D: DrawTarget<Color = Self::Color>,
                {
                    let first = self
                        .first
                        .colors()
                        .into_iter()
                        .map(|color| self.colormap.get(color));

                    let second = self
                        .second
                        .colors()
                        .into_iter()
                        .map(|color| self.colormap.get(color));

                    let start = self.colormap.first();
                    let end = self.colormap.last();
                    if start == end {
                        target.fill_solid(&self.area, start)
                    } else {
                        let colors = first
                            .zip(second)
                            .map(|(first, second)| first.screen(second, start, end));

                        target.fill_contiguous(&self.area, colors)
                    }
                }
            }
        )*
    }
}

impl_drawable! {
    BinaryColor, 2,
    Gray2, 4,
    Gray4, 16,
    Gray8, 256,
}

macro_rules! impl_mixed {
    (
        $(
            $color_type:ty, $array_length:literal,
        )*
    ) => {
        $(
            impl<U> Mixed<U, $color_type, $array_length> for Image<U>
            where
                U: ImageDrawable<Color = $color_type> + Colors<$color_type>,
            {
                fn mixed<'b, 'c, V, T>(
                    &self,
                    other: &'b Image<V>,
                    colormap: &'c Colormap<T, $array_length>,
                ) -> ImageMix<'_, 'b, 'c, U, V, T, $array_length>
                where
                    V: ImageDrawable<Color = $color_type> + Colors<$color_type>,
                    T: PixelColor + Default + Invert + Screen,
                {
                    let area = self.bounding_box().intersection(&other.bounding_box());
                    let first = self.clipped(&area);
                    let second = other.clipped(&area);

                    ImageMix::new(first, second, colormap, area)
                }
            }
        )*
    }
}

impl_mixed! {
    BinaryColor, 2,
    Gray2, 4,
    Gray4, 16,
    Gray8, 256,
}
