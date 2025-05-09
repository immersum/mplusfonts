# Changelog

## [Unreleased]

### Fixed

- The font metrics to match their definitions in the TrueType fonts. The `ascender` is not used but
  is a public field.

### Changed

- The `strings` attribute macro to visit token streams when gathering string literals. For example,
  parameters to any macro such as [`format!`] and [`concat!`] are token streams.
- Upgrade dependencies: `swash 0.2.4`.

[`format!`]: https://doc.rust-lang.org/std/macro.format.html
[`concat!`]: https://doc.rust-lang.org/core/macro.concat.html

## [0.1.4] - 2025-04-17

### Changed

- Improve `mplus!` macro expansion performance for when the `kern` helper is used.
- Implementation of font rasterization to be multithreaded; this can only have a noticeable effect
  when `positions` is greater than one.

## [0.1.3] - 2025-04-03

### Added

- The `kern` helper that can be used to create variable-width bitmap fonts with font-based kerning
  when populated using character ranges. This was previously only possible when specifying strings.

### Changed

- Upgrade dependencies: `defmt 1.0` and `swash 0.2.2`.

## [0.1.2] - 2025-03-28

### Fixed

- Rust version `1.86` not compiling the `mplusfonts` crate.

## [0.1.1] - 2025-03-21

### Fixed

- Standard ligatures such as _ff_ and _ffi_ appearing in monospaced text; this is now disabled.

## [0.1.0] - 2025-03-20

### Added

- The `strings` attribute macro and its `strings::skip` and `strings::emit` helper attributes.
- The `mplus!` function-like procedural macro for bitmap font generation using the `swash` crate.
- Implementation of the text renderer interface from the `embedded-graphics` crate.
- Settings for the text and background colors.
- Fonts by Coji Morishita.

[0.1.0]: https://github.com/immersum/mplusfonts/releases/tag/v0.1.0
[0.1.1]: https://github.com/immersum/mplusfonts/releases/tag/v0.1.1
[0.1.2]: https://github.com/immersum/mplusfonts/releases/tag/v0.1.2
[0.1.3]: https://github.com/immersum/mplusfonts/releases/tag/v0.1.3
[0.1.4]: https://github.com/immersum/mplusfonts/releases/tag/v0.1.4
