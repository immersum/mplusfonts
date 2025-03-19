use embedded_graphics::iterator::raw::RawDataSlice;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::pixelcolor::raw::BigEndian;

use crate::image::{Image, ImageRaw};

/// Image set, for looking up glyph images using sub-pixel offset data.
///
/// The length of the array is equal to the number of positions used during font rasterization.
///
/// The [`ImageSet::Array`] variant only appears in the data structures of variable-width bitmap
/// fonts, and even there, it is only used for glyphs that do not have square bounding boxes. For
/// kanji, kana, and for all glyphs in monospaced bitmap fonts, the [`ImageSet::Repeated`] variant
/// is used instead.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ImageSet<'a, C, const N: usize>
where
    C: PixelColor + From<C::Raw>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    /// A single repeated image referencing raw data.
    Repeated(Image<ImageRaw<'a, C>>),
    /// An array of images referencing raw data.
    Array([Image<ImageRaw<'a, C>>; N]),
}

impl<'a, C, const N: usize> ImageSet<'a, C, N>
where
    C: PixelColor + From<C::Raw>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    /// Returns either the single repeated image or the image at the specified index, wrapping if
    /// greater than or equal to the length of the array.
    pub const fn get(&self, index: usize) -> &Image<ImageRaw<'a, C>> {
        match self {
            Self::Repeated(image) => image,
            Self::Array(array) => &array[index % N],
        }
    }
}
