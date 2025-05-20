//! Color math for bitmap fonts.
//!
//! When a bitmap font is created, its pixels are assigned one of four possible color types:
//! [`BinaryColor`], [`Gray2`], [`Gray4`], or [`Gray8`]. This module provides the functions that
//! enable downsampling, color conversion from any [`GrayColor`] to any other color type that a
//! [`DrawTarget`](../../embedded_graphics_core/draw_target/trait.DrawTarget.html) expects,
//! applying color settings, and mixing colors in [`Screen`] blend mode.

use core::array;

use embedded_graphics::pixelcolor::*;

/// Array of colors having type `T`, for lookup-table-based color conversion.
///
/// The length of the array is equal to the number of gray values that can be converted.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Colormap<T: Copy, const N: usize>([T; N]);

/// Linear gradient definition.
///
/// A colormap that implements this trait can be created from two colors of type `T`, which become
/// the first and the last element in the resulting array, while the colors in between are
/// calculated using a linear equation.
pub trait Linear<T> {
    /// Returns a linear gradient with the specified start and end colors.
    fn linear(start: T, end: T) -> Self;
}

/// Color inversion.
///
/// A color that implements this trait can be changed into its negative self.
pub trait Invert {
    /// Returns the inverse of the specified color.
    fn invert(self) -> Self;
}

/// Screen blend mode with color conversion.
///
/// A color that implements this trait can be mixed with another color of the same type,
/// multiplying their inverse color components and inverting the resulting color, which may only
/// shift a given color away from the start color and towards the end color.
pub trait Screen {
    /// Converts the first two color values to the range of colors that is defined by the specified
    /// start and end colors, mixing the intermediate colors and returning the final result.
    fn screen(self, other: Self, start: Self, end: Self) -> Self;
}

impl<T: Copy, const N: usize> Colormap<T, N> {
    /// Returns the first element of the colormap.
    pub const fn first(&self) -> T {
        let Colormap(array) = self;

        array[0]
    }

    /// Returns the last element of the colormap.
    pub const fn last(&self) -> T {
        let Colormap(array) = self;

        array[N - 1]
    }
}

macro_rules! impl_colormap {
    (
        $(
            $color_ident:ident, $color_type:ty, $array_length:literal, $into_index:expr,
        )*
    ) => {
        $(
            impl<T: Copy> Colormap<T, $array_length> {
                /// Returns the color that is mapped to the specified gray value.
                pub fn get(&self, $color_ident: $color_type) -> T {
                    let Colormap(array) = self;
                    let index: usize = $into_index.into();

                    array[index % $array_length]
                }
            }
        )*
    }
}

impl_colormap! {
    color, BinaryColor, 2, color.is_on(),
    color, Gray2, 4, color.luma(),
    color, Gray4, 16, color.luma(),
    color, Gray8, 256, color.luma(),
}

const fn convert_channel<const N: usize>(value: u8, start: u8, end: u8) -> u8 {
    const SHIFT: usize = 23;
    const CONST_0_5: i32 = 1 << (SHIFT - 1);

    let diff = end as i32 - start as i32;
    let a = (diff << SHIFT) / (N - 1) as i32;
    let b = (start as i32) << SHIFT;
    let result = a * value as i32 + b + CONST_0_5;

    (result >> SHIFT) as u8
}

macro_rules! impl_linear_rgb {
    ($($rgb_type:ty),+) => {
        $(
            impl<const N: usize> Linear<$rgb_type> for Colormap<$rgb_type, N> {
                fn linear(start: $rgb_type, end: $rgb_type) -> Self {
                    let colors = array::from_fn(|index| {
                        let r = convert_channel::<N>(index as u8, start.r(), end.r());
                        let g = convert_channel::<N>(index as u8, start.g(), end.g());
                        let b = convert_channel::<N>(index as u8, start.b(), end.b());

                        <$rgb_type>::new(r, g, b)
                    });

                    Self(colors)
                }
            }
        )*
    }
}

impl_linear_rgb!(
    Rgb555, Bgr555, Rgb565, Bgr565, Rgb666, Bgr666, Rgb888, Bgr888
);

macro_rules! impl_linear_gray {
    ($($gray_type:ty),+) => {
        $(
            impl<const N: usize> Linear<$gray_type> for Colormap<$gray_type, N> {
                fn linear(start: $gray_type, end: $gray_type) -> Self {
                    let colors = array::from_fn(|index| {
                        let luma = convert_channel::<N>(index as u8, start.luma(), end.luma());

                        <$gray_type>::new(luma)
                    });

                    Self(colors)
                }
            }
        )*
    }
}

impl_linear_gray!(Gray2, Gray4, Gray8);

impl<const N: usize> Linear<BinaryColor> for Colormap<BinaryColor, N> {
    fn linear(start: BinaryColor, end: BinaryColor) -> Self {
        let colors = array::from_fn(|index| if index < N / 2 { start } else { end });

        Self(colors)
    }
}

macro_rules! impl_invert_rgb {
    ($($rgb_type:ty),+) => {
        $(
            impl Invert for $rgb_type {
                fn invert(self) -> Self {
                    let r = <$rgb_type>::MAX_R - self.r();
                    let g = <$rgb_type>::MAX_G - self.g();
                    let b = <$rgb_type>::MAX_B - self.b();

                    Self::new(r, g, b)
                }
            }
        )*
    }
}

impl_invert_rgb!(
    Rgb555, Bgr555, Rgb565, Bgr565, Rgb666, Bgr666, Rgb888, Bgr888
);

impl Invert for Gray2 {
    fn invert(self) -> Self {
        Self::new(0b00000011 - self.luma())
    }
}

impl Invert for Gray4 {
    fn invert(self) -> Self {
        Self::new(0b00001111 - self.luma())
    }
}

impl Invert for Gray8 {
    fn invert(self) -> Self {
        Self::new(0b11111111 - self.luma())
    }
}

impl Invert for BinaryColor {
    fn invert(self) -> Self {
        self.invert()
    }
}

const fn screen_mix_channel(first: u8, second: u8, start: u8, end: u8) -> u8 {
    const SHIFT: usize = 15;
    const CONST_0_5: i32 = 1 << (SHIFT - 1);

    if start == end {
        return start;
    }

    let diff = end as i32 - start as i32;
    let first = end as i32 - first as i32;
    let second = end as i32 - second as i32;
    let product = first * ((second << SHIFT) + CONST_0_5);
    let minuend = ((end as i32) << SHIFT) + CONST_0_5;
    let result = minuend - product / diff;

    (result >> SHIFT) as u8
}

macro_rules! impl_screen_mix_rgb {
    ($($rgb_type:ty),+) => {
        $(
            impl Screen for $rgb_type {
                fn screen(self, other: Self, start: Self, end: Self) -> Self {
                    let r = screen_mix_channel(self.r(), other.r(), start.r(), end.r());
                    let g = screen_mix_channel(self.g(), other.g(), start.g(), end.g());
                    let b = screen_mix_channel(self.b(), other.b(), start.b(), end.b());

                    <$rgb_type>::new(r, g, b)
                }
            }
        )*
    }
}

impl_screen_mix_rgb!(
    Rgb555, Bgr555, Rgb565, Bgr565, Rgb666, Bgr666, Rgb888, Bgr888
);

macro_rules! impl_screen_mix_gray {
    ($($gray_type:ty),+) => {
        $(
            impl Screen for $gray_type {
                fn screen(self, other: Self, start: Self, end: Self) -> Self {
                    let luma = screen_mix_channel(self.luma(), other.luma(), start.luma(), end.luma());

                    <$gray_type>::new(luma)
                }
            }
        )*
    }
}

impl_screen_mix_gray!(Gray2, Gray4, Gray8);

impl Screen for BinaryColor {
    fn screen(self, other: Self, start: Self, end: Self) -> Self {
        if self == end || other == end {
            end
        } else {
            start
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_convert_channel {
        (
            $(
                $fn_ident:ident, $n:expr, $value:expr, $start:expr, $end:expr, $expected:expr,
            )*
        ) => {
            $(
                #[test]
                fn $fn_ident() {
                    let result = convert_channel::<$n>($value, $start, $end);
                    assert_eq!(result, $expected);
                }
            )*
        }
    }

    test_convert_channel! {
        convert_8bpp_255_to_0_255, { 2usize.pow(8) }, 255, 0, 255, 255,
        convert_8bpp_255_to_0_128, { 2usize.pow(8) }, 255, 0, 128, 128,
        convert_8bpp_128_to_0_128, { 2usize.pow(8) }, 128, 0, 128, 64,
        convert_8bpp_128_to_0_64, { 2usize.pow(8) }, 128, 0, 64, 32,
        convert_8bpp_64_to_0_64, { 2usize.pow(8) }, 64, 0, 64, 16,

        convert_8bpp_255_to_255_0, { 2usize.pow(8) }, 255, 255, 0, 0,
        convert_8bpp_255_to_255_128, { 2usize.pow(8) }, 255, 255, 128, 128,
        convert_8bpp_128_to_255_128, { 2usize.pow(8) }, 128, 255, 128, 255 - 128 / 2,
        convert_8bpp_128_to_255_64, { 2usize.pow(8) }, 128, 255, 64, 255 - 128 / 4 * 3,
        convert_8bpp_64_to_255_64, { 2usize.pow(8) }, 64, 255, 64, 255 - 64 / 4 * 3,

        convert_5bpp_31_to_0_255, { 2usize.pow(5) }, 31, 0, 255, 255,
        convert_4bpp_15_to_0_255, { 2usize.pow(4) }, 15, 0, 255, 255,
        convert_3bpp_7_to_0_255, { 2usize.pow(3) }, 7, 0, 255, 255,
        convert_2bpp_3_to_0_255, { 2usize.pow(2) }, 3, 0, 255, 255,
        convert_1bpp_1_to_0_255, { 2usize.pow(1) }, 1, 0, 255, 255,
    }

    macro_rules! test_screen_mix_channel {
        (
            $(
                $fn_ident:ident,
                $first:expr,
                $second:expr,
                $start:expr,
                $end:expr,
                $expected:expr,
            )*
        ) => {
            $(
                #[test]
                fn $fn_ident() {
                    let result = screen_mix_channel($first, $second, $start, $end);
                    assert_eq!(result, $expected);
                }
            )*
        }
    }

    test_screen_mix_channel! {
        screen_mix_channel_255_255_on_0_255, 255, 255, 0, 255, 255,
        screen_mix_channel_128_128_on_0_255, 128, 128, 0, 255, 128 + 128 / 2,
        screen_mix_channel_64_64_on_0_255, 64, 64, 0, 255, 64 + 64 / 4 * 3,
        screen_mix_channel_32_32_on_0_255, 32, 32, 0, 255, 32 + 32 / 8 * 7,
        screen_mix_channel_0_0_on_0_255, 0, 0, 0, 255, 0,

        screen_mix_channel_255_255_on_255_0, 255, 255, 255, 0, 255,
        screen_mix_channel_224_224_on_255_0, 224, 224, 255, 0, 224 - 224 / 8,
        screen_mix_channel_192_192_on_255_0, 192, 192, 255, 0, 192 - 192 / 4,
        screen_mix_channel_128_128_on_255_0, 128, 128, 255, 0, 128 - 128 / 2,
        screen_mix_channel_0_0_on_255_0, 0, 0, 255, 0, 0,

        screen_mix_channel_255_0_on_0_255, 255, 0, 0, 255, 255,
        screen_mix_channel_128_64_on_0_255, 128, 64, 0, 255, 128 + 64 / 2,
        screen_mix_channel_128_32_on_0_255, 128, 32, 0, 255, 128 + 32 / 2,
        screen_mix_channel_64_128_on_0_255, 64, 128, 0, 255, 64 + 128 / 4 * 3,
        screen_mix_channel_32_128_on_0_255, 32, 128, 0, 255, 32 + 128 / 8 * 7,

        screen_mix_channel_255_0_on_255_0, 255, 0, 255, 0, 0,
        screen_mix_channel_224_192_on_255_0, 224, 192, 255, 0, 224 / 4 * 3,
        screen_mix_channel_224_128_on_255_0, 224, 128, 255, 0, 224 / 2,
        screen_mix_channel_192_224_on_255_0, 192, 224, 255, 0, 192 / 8 * 7,
        screen_mix_channel_128_224_on_255_0, 128, 224, 255, 0, 128 / 8 * 7,

        screen_mix_channel_128_128_on_0_128, 128, 128, 0, 128, 128,
        screen_mix_channel_64_64_on_0_128, 64, 64, 0, 128, 64 + 64 / 2,
        screen_mix_channel_64_64_on_0_64, 64, 64, 0, 64, 64,
        screen_mix_channel_32_32_on_0_64, 32, 32, 0, 64, 32 + 32 / 2,
        screen_mix_channel_32_32_on_0_32, 32, 32, 0, 32, 32,

        screen_mix_channel_128_128_on_128_0, 128, 128, 128, 0, 128,
        screen_mix_channel_128_64_on_128_0, 128, 64, 128, 0, 64,
        screen_mix_channel_128_32_on_128_0, 128, 32, 128, 0, 32,
        screen_mix_channel_64_128_on_128_0, 64, 128, 128, 0, 64,
        screen_mix_channel_32_128_on_128_0, 32, 128, 128, 0, 32,

        screen_mix_channel_255_255_on_255_255, 255, 255, 255, 255, 255,
        screen_mix_channel_255_255_on_128_128, 255, 255, 128, 128, 128,
        screen_mix_channel_128_128_on_128_128, 128, 128, 128, 128, 128,
        screen_mix_channel_0_0_on_128_128, 0, 0, 128, 128, 128,
        screen_mix_channel_0_0_on_0_0, 0, 0, 0, 0, 0,
    }
}
