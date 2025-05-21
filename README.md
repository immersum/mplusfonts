# mplusfonts

Parametrized bitmap fonts for [`embedded-graphics`], with font rasterization powered by [`swash`].

Adds an *optimal* subset of [**M<sup>+</sup> FONTS**] to your next embedded Rust project.[^1] 

![Screenshot with English and Japanese text](assets/mango-screenshot.png "Font definitions:
`mplus!(code(120), 500, 15, false, 1, 8)`
`mplus!(2, 800, 9, true, 16, 8)`")

[^1]: Rust 2024 is required to build this crate

## Features

- **A family of variable-width and monospaced fonts** - [`mplusfonts`] allows you to choose between
  three typefaces, with font properties that you will want to specify. Using this crate, you can
  have font anti-aliasing and font-based kerning on embedded devices, in a `no_std` environment.
- **No font files needed in your project** - The [`mplus!`] macro generates Rust code for your
  bitmap font. After expanding the macro and compiling your code, the TrueType fonts that come with
  this crate are no longer used.
- **Includes only the glyphs that you want** - The [`strings`] attribute helps you find and add
  characters and character clusters to your bitmap font. You can also specify ranges of characters
  to include as parameters for the `mplus!` macro.
- **Japanese scripts** - Designed by Coji Morishita and licensed under the SIL Open Fonts License
  ([LICENSE]), **M<sup>+</sup> FONTS** has support for over 5 700 kanji glyphs.[^2] Since this
  crate is based on **M<sup>+</sup> FONTS**, you gain access to all of its features. 

[**M<sup>+</sup> FONTS**]: https://mplusfonts.github.io/
[`embedded-graphics`]: https://crates.io/crates/embedded-graphics
[`swash`]: https://crates.io/crates/swash
[`mplusfonts`]: https://crates.io/crates/mplusfonts
[`mplus!`]: https://docs.rs/mplusfonts/latest/mplusfonts/macro.mplus.html
[`strings`]: https://docs.rs/mplusfonts/latest/mplusfonts/attr.strings.html
[LICENSE]: macros/fonts/LICENSE

[^2]: <https://mplusfonts.github.io/#variable>

## Usage

1. Make sure you have added `mplusfonts` as a dependency.
2. Enable static text rendering by applying an attribute to your function with string definitions.
3. Create a bitmap font *inside* your function so that `#[strings]` can find its helper attributes.
4. Apply `#[strings::emit]` to the bitmap font definition.
5. Include any additional character ranges in the bitmap font that you need. 
6. Have a character style use the bitmap font.
7. You can now start drawing text.
8. Exclude any string literals in your function that are not drawn by using `#[strings::skip]`.

## Examples

```toml
[dependencies]
mplusfonts = "0.2"
```

```rust
#[mplusfonts::strings]
pub fn main() -> Result<(), Infallible> {
    let text = format!("{} KB_OK", 16 * 40);

    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(320, 240));

    #[strings::emit]
    let bitmap_font = mplus!(code(115), BOLD, code_line_height(24), true, 1, 4, '0'..='9');

    let character_style = BitmapFontStyle::new(&bitmap_font, Rgb565::GREEN);

    Text::new(&text, Point::new(20, 20), character_style).draw(&mut display)?;

    let output_settings = OutputSettingsBuilder::new().scale(3).build();

    #[strings::skip]
    Window::new("Simulator", &output_settings).show_static(&display);

    Ok(())
}
```

For more examples, see the [examples] folder.

## Limitations

* Rendering [combining characters](https://en.wikipedia.org/wiki/Combining_Diacritical_Marks) with
  characters for which no single code point exists, is a hit-or-miss.
* **Transparent backgrounds are not supported.** Alpha compositing is not available; this crate
  does not have an `alloc` feature.

[examples]: examples
[`BitmapFontStyle`]: https://docs.rs/mplusfonts/latest/mplusfonts/style/struct.BitmapFontStyle.html

## Minimum supported Rust version

The minimum supported Rust version for `mplusfonts` is `1.85`.

## License

The source code of `mplusfonts` is dual-licensed under:

* Apache License, Version 2.0 ([LICENSE-APACHE] or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT License ([LICENSE-MIT] or <http://opensource.org/licenses/MIT>)

at your option.

[LICENSE-APACHE]: LICENSE-APACHE
[LICENSE-MIT]: LICENSE-MIT
