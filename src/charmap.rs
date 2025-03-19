use embedded_graphics::iterator::raw::RawDataSlice;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::pixelcolor::raw::BigEndian;

use crate::glyph::Glyph;

/// Key that is unique to a charmap entry in a bitmap font.
pub type CharmapEntryKey<'a> = &'a str;

/// Charmap entry.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CharmapEntry<'a, C, const N: usize>
where
    C: PixelColor + From<C::Raw>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    /// The key for the charmap entry.
    pub key: CharmapEntryKey<'a>,
    /// The value for advancing the position in the text run for the charmap entry.
    pub advance_chars: usize,
    /// A function that takes the key for the next charmap entry, returning the value for advancing
    /// the position along the _x_-axis for the charmap entry in order to get to the position of
    /// the next charmap entry.
    pub advance_width_to: fn(CharmapEntryKey<'a>) -> f32,
    /// The glyph.
    pub glyph: Glyph<'a, C, N>,
}

/// Charmap, for looking up glyph data, matching as many characters as possible at a time.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Charmap<'a, C, const N: usize>
where
    C: PixelColor + From<C::Raw>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    /// A leaf node with a single charmap entry.
    Leaf(CharmapEntry<'a, C, N>),
    /// A branch node with a function that takes the next character to match, returning another
    /// charmap.
    Branch(fn(char) -> &'a Charmap<'a, C, N>),
}

impl<'a, C, const N: usize> CharmapEntry<'a, C, N>
where
    C: PixelColor + From<C::Raw>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    /// Charmap entry for the invisible glyph.
    pub const NULL: Self = Self {
        key: "",
        advance_chars: 0,
        advance_width_to: |_| 0.0,
        glyph: Glyph::NULL,
    };
}

impl<'a, C, const N: usize> Charmap<'a, C, N>
where
    C: PixelColor + From<C::Raw>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    /// Finds the charmap entry for a given input, traversing the branch nodes while also matching
    /// characters from the specified string slice until a leaf node is found.
    pub fn get(&self, slice: &str) -> &CharmapEntry<'a, C, N> {
        let mut chars = slice.chars();
        let first = chars.next().unwrap_or_default();
        let slice = chars.as_str();

        match self {
            Self::Leaf(entry) => entry,
            Self::Branch(map) => map(first).get(slice),
        }
    }
}
