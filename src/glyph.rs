//! Glyphs and glyph clusters.
//!
//! Glyphs are the visual elements of a character. There is a one-to-one mapping for almost all of
//! what [`mplusfonts`](../index.html) supports --- what looks like a single character, is indeed a
//! single-glyph character. Combining characters are supported as a best effort; these are known to
//! display artifacts when rendered, but a less complex glyph cluster such as _g̈́_ does appear as
//! expected.

use embedded_graphics::iterator::raw::RawDataSlice;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::pixelcolor::raw::BigEndian;

use crate::image::{Image, ImageSet};

/// Glyph identifier.
pub type GlyphId = u16;

/// Glyph that may have references to additional glyphs, forming a glyph cluster.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Glyph<'a, C, const N: usize>
where
    C: PixelColor + From<C::Raw>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    /// The glyph identifier.
    pub id: GlyphId,
    /// The image set.
    pub images: ImageSet<'a, C, N>,
    /// The next glyph.
    pub next: Option<&'a NextGlyph<'a, C, N>>,
}

/// Glyph as part of a glyph cluster.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NextGlyph<'a, C, const N: usize>
where
    C: PixelColor + From<C::Raw>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    /// The _x_-offset from the position of the next glyph cluster.
    pub x_offset: f32,
    /// The _y_-offset from the position of the next glyph cluster.
    pub y_offset: f32,
    /// The glyph.
    pub glyph: Glyph<'a, C, N>,
}

impl<'a, C, const N: usize> Glyph<'a, C, N>
where
    C: PixelColor + From<C::Raw>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    /// The invisible glyph.
    pub const NULL: Self = Self {
        id: 0,
        images: ImageSet::Repeated(Image::NULL),
        next: None,
    };
}
