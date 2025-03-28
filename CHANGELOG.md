# Changelog

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
