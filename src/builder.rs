use embedded_graphics::iterator::raw::RawDataSlice;
use embedded_graphics::pixelcolor::raw::BigEndian;
use embedded_graphics::pixelcolor::{BinaryColor, PixelColor};

use crate::color::{Invert, Screen};
use crate::font::BitmapFont;
use crate::style::BitmapFontStyle;

/// Builder for a style using a bitmap font.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BitmapFontStyleBuilder<'a, 'b, T, C, const N: usize>
where
    C: PixelColor + From<C::Raw>,
    T: PixelColor + Default + Invert + Screen,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    style: BitmapFontStyle<'a, 'b, T, C, N>,
}

impl<'a, 'b, T, C, const N: usize> BitmapFontStyleBuilder<'a, 'b, T, C, N>
where
    C: PixelColor + From<C::Raw>,
    T: PixelColor + Default + Invert + Screen,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    /// Resets the text color to the default value for the style.
    pub const fn reset_text_color(mut self) -> Self {
        self.style.text_color = None;
        self
    }

    /// Resets the background color to the default value for the style.
    pub const fn reset_background_color(mut self) -> Self {
        self.style.background_color = None;
        self
    }

    /// Sets the text color to the specified value.
    pub const fn text_color(mut self, text_color: T) -> Self {
        self.style.text_color = Some(text_color);
        self
    }

    /// Sets the background color to the specified value.
    pub const fn background_color(mut self, background_color: T) -> Self {
        self.style.background_color = Some(background_color);
        self
    }

    /// Consumes the builder, returning a new one that is using the specified bitmap font.
    pub const fn font<'z, D, const M: usize>(
        self,
        font: &'z BitmapFont<'a, D, M>,
    ) -> BitmapFontStyleBuilder<'a, 'z, T, D, M>
    where
        D: PixelColor + From<D::Raw>,
        RawDataSlice<'a, D::Raw, BigEndian>: IntoIterator<Item = D::Raw>,
    {
        BitmapFontStyleBuilder {
            style: BitmapFontStyle {
                font,
                text_color: self.style.text_color,
                background_color: self.style.background_color,
            },
        }
    }

    /// Consumes the builder, returning the style.
    pub const fn build(self) -> BitmapFontStyle<'a, 'b, T, C, N> {
        self.style
    }
}

impl<T> BitmapFontStyleBuilder<'_, '_, T, BinaryColor, 0>
where
    T: PixelColor + Default + Invert + Screen,
{
    /// Creates a new, empty builder using text and background colors of type `T`.
    pub const fn new() -> Self {
        Self {
            style: BitmapFontStyle {
                font: &BitmapFont::NULL,
                text_color: None,
                background_color: None,
            },
        }
    }
}

impl<'a, T, C, const N: usize> Default for BitmapFontStyleBuilder<'a, '_, T, C, N>
where
    C: PixelColor + From<C::Raw>,
    T: PixelColor + Default + Invert + Screen,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    fn default() -> Self {
        Self {
            style: BitmapFontStyle {
                font: &BitmapFont::NULL,
                text_color: None,
                background_color: None,
            },
        }
    }
}
