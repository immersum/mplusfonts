//! This crate contains **M<sup>+</sup> FONTS**, [a font family](https://mplusfonts.github.io/) by
//! Coji Morishita; it is a dependency of [`mplusfonts`](../mplusfonts/index.html), with font
//! rasterization powered by [`swash`].

mod mplus;
mod strings;

use proc_macro::TokenStream;
use syn::{meta, parse_macro_input};

/// Collects string literals for rewriting [`mplus!`] macro invocations prior to their expansion.
///
/// This attribute should be applied to the item that contains both the string literals that appear
/// in [`Text`](../embedded_graphics/text/struct.Text.html) drawables and the [`mplus!`] macro that
/// provides the bitmap font in the [`Text`](../embedded_graphics/text/struct.Text.html)'s
/// character style.
///
/// Use `#[strings::skip]` to exclude string literals or blocks from the output of
/// [`macro@strings`], and apply `#[strings::emit]` to the macro invocations that you want to
/// modify to have additional input --- the string literals that have been collected; appended as a
/// single slice literal expression.
///
/// # Examples
///
/// ```
/// # use core::convert::Infallible;
/// #
/// # use embedded_graphics::pixelcolor::Rgb888;
/// # use embedded_graphics::prelude::*;
/// # use embedded_graphics::text::Text;
/// # use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
/// # use mplusfonts::mplus;
/// # use mplusfonts::style::BitmapFontStyle;
/// #
/// #[mplusfonts::strings]
/// pub fn main() -> Result<(), Infallible> {
///     let mut display: SimulatorDisplay<Rgb888> = SimulatorDisplay::new(Size::new(120, 120));
///
///     #[strings::emit]
///     let bitmap_font = mplus!(2, 480, 16, true, 4, 8, /* will inject ["It works!"] here */);
///
///     let character_style = BitmapFontStyle::new(&bitmap_font, Rgb888::new(0, 210, 255));
///
///     Text::new("It works!", Point::new(0, 120), character_style).draw(&mut display)?;
///
///     let output_settings = OutputSettingsBuilder::new().scale(6).pixel_spacing(2).build();
///
///     #[strings::skip]
///     Window::new("Simulator", &output_settings).show_static(&display);
///
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn strings(args: TokenStream, input: TokenStream) -> TokenStream {
    let message = "remove the arguments to the attribute";
    let parser = meta::parser(|meta| Err(meta.error(message)));
    parse_macro_input!(args with parser);

    let item = parse_macro_input!(input as syn::Item);
    strings::strings_impl(item).into()
}

/// Produces a struct expression for creating a
/// [`BitmapFont`](../mplusfonts/struct.BitmapFont.html).
///
/// The generated data structure is a single self-contained expression with static references and
/// lookup-table-like structures that point to kerning and pixel information, image offset data,
/// etc. The individual values appear in the form of number and byte slice literals after macro
/// expansion, so the usage of this macro is comparable to using [`include_bytes!`] with a
/// bitmap font, but without creating any additional files in the build process.
///
/// The data types live in the [`mplusfonts`](../mplusfonts/index.html) crate, which also features
/// [`BitmapFontStyle`](../mplusfonts/struct.BitmapFontStyle.html), the intended consumer of the
/// generated data.
///
/// [`include_bytes!`]: https://doc.rust-lang.org/core/macro.include_bytes.html
///
/// # Arguments
///
/// * `font` - Typeface and font width. Specify `1` or `2` to use the respective variable-width
///   **M<sup>+</sup>** font or, for a monospaced font, specify `code`, which takes a `width`
///   parameter and uses **M<sup>+</sup> Code Latin 50/60**, falling back to **M<sup>+</sup> 1**
///   for glyphs that are not parametrized by width.
///   * `code(width)` - Font width. Ranges from `100` to `125`. Only available as a parameter to
///     `code`.
/// * `weight` - Font weight. Ranges from `100` to `900`. Capped at `700` for `code`.
/// * `size` - Font size. Specify either as a value in pixels per _em_-size or, for convenience,
///   specify one of helpers listed here, all of which take a `px` parameter, performing a
///   conversion to pixels per _em_-size. In both cases, any `.0` can be omitted.
///   * `x_height(px)` - Height of the small letter _x_.
///   * `cap_height(px)` - Height of capital letters.
///   * `line_height(px)` - Line height used for `1` and `2`.
///   * `code_line_height(px)` - Line height used for `code`.
/// * `hint` - Font hinting. Set to [`true`] to enable or [`false`] to disable this feature.
///   Improves the clarity of fonts at small sizes at the cost of glyphs becoming less proportional
///   along the _y_-axis.
/// * `positions` - Number of glyph images, one for each sub-pixel offset. Ranges from `1` to `16`.
///   Specify `1` for a single image at `.0` offset. Ignored for glyphs with square bounding
///   boxes such as kanji, kana, and also forced to `1` for `code`.
/// * `bit_depth` - Bit depth of glyph images. Specify `n` to use _2_<sup>`n`</sup> values of gray.
///   Limited to `1`, `2`, `4`, `8`.
/// * `sources` - Sources of characters for feeding the glyph shaper. Enable support for rendering
///   the individual strings here; otherwise, this instance returns boxes (image representations of
///   `.notdef`) when looking up glyph data.
///   * Ranges of character literals. Use this option for arbitrary strings created at runtime.
///     * `range` - All characters in the specified range so long as **M<sup>+</sup>** provides the
///       glyphs. Recommended for including digits, kana, and for letters in monospaced fonts only.
///     * `kern(range, slice)` - The same characters as with separate `range` and `slice` sources,
///       but with kerning information included with each character in order to cover all possible
///       sequences of characters, interleaved with the specified strings. The `kern` function-like
///       helper takes two parameters, the first is a range of character literals, and the second
///       is an array of string literals. Recommended for populating variable-width bitmap fonts
///       with letters in Latin-script alphabets.
///   * Arrays of string literals. Specify all static text in any order, grouped in any manner.
///     * `slice` - All characters that occur in the specified strings. Recommended when the text
///       fragments to be rendered are known at compile-time. Only includes the kerning information
///       for the sequences of characters as they appear. Part of the [`macro@strings`]
///       infrastructure.
///
/// The optional `sources` argument makes this a variadic-function-like procedural macro.
///
/// # Aliases
///
/// Built-in constant-like identifiers can be substituted for common weight and width values.
///
/// | Weight Name           | Value |
/// |-----------------------|-------|
/// | `THIN`                | `100` |
/// | `EXTRA_LIGHT`         | `200` |
/// | `LIGHT`               | `300` |
/// | `NORMAL` or `REGULAR` | `400` |
/// | `MEDIUM`              | `500` |
/// | `SEMI_BOLD`           | `600` |
/// | `BOLD`                | `700` |
/// | `EXTRA_BOLD`          | `800` |
/// | `BLACK`               | `900` |
///
/// | Width Name | Value |
/// |------------|-------|
/// | `NORMAL`   | `100` |
/// | `EXPANDED` | `125` |
///
/// # Examples
///
/// ```
/// # use mplusfonts_macros::mplus;
/// #
/// mplus!(1, 750, x_height(5), false, 2, 4, ["Yes", "No"]);
/// mplus!(1, 525, cap_height(7), false, 2, 4, ["キャンセル"]);
/// mplus!(2, BOLD, line_height(20), false, 2, 4, ["Tokyo"], ["東京"]);
/// mplus!(2, 575, line_height(20), true, 4, 4, kern(' '..='ȷ', ["ffi", "ffl"]));
/// mplus!(code(100), SEMI_BOLD, 18, true, 1, 4, '0'..='9', [",.-"]);
/// mplus!(code(125), 480, 13.5, true, 1, 4, 'A'..='Z', 'ぁ'..='ゖ');
/// ```
///
/// The amount of flash memory (storage space for `.rodata`) that is going to be used, will be a
/// few **kilobytes**. In all of these examples, specifying `..` for `sources` and including all
/// characters would change the size of each bitmap font to approximately _2_–_3_ **megabytes**.
///
/// ## All-inclusive bitmap font
///
/// The following examples produce binary outputs that are approximately _16_ megabytes in size:
///
/// ```
/// # use mplusfonts_macros::mplus;
/// #
/// mplus!(1, 500, 50, true, 4, 8, .., ["ff", "fi", "ffi", "fl", "ffl"]);
/// mplus!(1, 500, 50, true, 4, 8, kern(.., ["ffi", "ffl"]));
/// ```
#[proc_macro]
pub fn mplus(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as mplus::Arguments);
    mplus::mplus_impl(args).into()
}
