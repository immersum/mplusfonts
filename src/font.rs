use embedded_graphics::iterator::raw::RawDataSlice;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::pixelcolor::raw::BigEndian;

use crate::charmap::{Charmap, CharmapEntry};
use crate::metrics::BitmapFontMetrics;

/// Bitmap font.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BitmapFont<'a, C, const N: usize>
where
    C: PixelColor + From<C::Raw>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    /// The charmap that holds all glyph data.
    pub charmap: Charmap<'a, C, N>,
    /// The metrics that are scaled to go with the bitmap font.
    pub metrics: BitmapFontMetrics,
}

impl<'a, C, const N: usize> BitmapFont<'a, C, N>
where
    C: PixelColor + From<C::Raw>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    /// The invisible bitmap font.
    pub const NULL: Self = Self {
        charmap: Charmap::Leaf(CharmapEntry::NULL),
        metrics: BitmapFontMetrics::NULL,
    };
}
