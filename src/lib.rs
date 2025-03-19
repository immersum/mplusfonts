//! **M<sup>+</sup> FONTS** for use with [`embedded-graphics`](embedded_graphics); this crate is
//! compatible with `no_std` and brings [a font family](https://mplusfonts.github.io/) by Coji
//! Morishita into the embedded Rust ecosystem, including the variable-width **M<sup>+</sup> 1**,
//! **M<sup>+</sup> 2**, and the monospaced **M<sup>+</sup> Code** typefaces.
//!
//! # Example of a bitmap font
//!
//! Font rasterization is achieved using [`mplusfonts-macros`](mplusfonts_macros), providing access
//! to over 5 700 kanji glyphs.[^1] By default, [`mplus!`] expands to an empty bitmap font, so even
//! basic characters such as digits and letters in Latin-script alphabets need to be specified in
//! order for them to end up as pixel information in the generated data structure. For example, to
//! create a fixed-width bitmap font that has support for rendering string representations of
//! values `0x00` through `0xFF`:
//!
//! ```
//! # use mplusfonts::mplus;
//! #
//! let bitmap_font = mplus!(code(100), 500, 12, false, 1, 8, '0'..='9', 'A'..='F', ["x"]);
//! ```
//!
//! * `code(100)` - Use the monospaced **M<sup>+</sup> Code** typeface; this is a variable font, so
//!   set its width-axis position to 100 percent (corresponds to **M<sup>+</sup> Code Latin 50**).
//! * `500` - Set the font weight to 500 (equivalent to `MEDIUM`).
//! * `12` - Set the font size to 12 pixels per _em_-size.
//! * `false` - Disable font hinting. Enable to force-align the top, the bottom, and the segments
//!   of glyph outlines that are running in parallel with the _x_-axis to the pixel grid.
//! * `1` - Set the quantization level for positions per pixel to 1 (value not used with
//!   **M<sup>+</sup> Code** --- no sub-pixel offsets required).
//! * `8` - Set the quantization level for gray values to 256 (8 bits per pixel).
//! * `'0'..='9'`, `'A'..='F'`, `["x"]` - Enroll the characters and strings that comprise the text
//!   to be rendered.
//!
//! <div class="warning">
//!   Character ranges are intended to be used for including digits and, in general, when all
//!   strings using a set of characters need to be made renderable; otherwise, you end up with a
//!   higher than optimal amount of data. See the example below for the recommended way to use this
//!   crate.
//! </div>
//!
//! [^1]: <https://mplusfonts.github.io/#variable>
//!
//! # Example of rendering static text
//!
//! Which characters a given instance of [`BitmapFont`] is required to include for rendering static
//! text is deterministic. To cover such cases, this crate provides the [`strings`] attribute and
//! two helper attributes.
//!
//! ```
//! # use core::convert::Infallible;
//! #
//! # use embedded_graphics::pixelcolor::Rgb888;
//! # use embedded_graphics::prelude::*;
//! # use embedded_graphics::text::Text;
//! # use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
//! # use mplusfonts::mplus;
//! # use mplusfonts::style::BitmapFontStyle;
//! #
//! #[mplusfonts::strings]
//! pub fn main() -> Result<(), Infallible> {
//!     let mut display: SimulatorDisplay<Rgb888> = SimulatorDisplay::new(Size::new(120, 120));
//!
//!     #[strings::emit]
//!     let bitmap_font = mplus!(2, 480, 16, true, 4, 8, /* will inject ["It works!"] here */);
//!
//!     let character_style = BitmapFontStyle::new(&bitmap_font, Rgb888::new(0, 210, 255));
//!
//!     Text::new("It works!", Point::new(0, 120), character_style).draw(&mut display)?;
//!
//!     let output_settings = OutputSettingsBuilder::new().scale(6).pixel_spacing(2).build();
//!
//!     #[strings::skip]
//!     Window::new("Simulator", &output_settings).show_static(&display);
//!
//!     Ok(())
//! }
//! ```
//!
//! For more examples, see the [examples](https://github.com/immersum/mplusfonts/examples)
//! directory on GitHub.

#![no_std]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]

mod adapter;
mod builder;
mod charmap;
mod font;
mod metrics;
mod rect;

pub mod color;
pub mod glyph;
pub mod image;
pub mod style;

pub use font::BitmapFont;

pub use charmap::*;
pub use metrics::*;

pub use mplusfonts_macros::mplus;
pub use mplusfonts_macros::strings;
