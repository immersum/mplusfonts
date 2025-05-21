use embedded_graphics::text::Baseline;

/// Metrics of a bitmap font.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BitmapFontMetrics {
    /// The top of the line, defined as _1160/1000 em_-size for **M<sup>+</sup> 1/2** and
    /// _1235/1000 em_-size for **M<sup>+</sup> Code**.
    pub top: f32,
    /// Typographic ascender, defined as _880/1000 em_-size for **M<sup>+</sup> 1/2** and
    /// _1000/1000 em_-size for **M<sup>+</sup> Code**.
    pub ascender: f32,
    /// The top of capital letters, defined as _730/1000 em_-size for both **M<sup>+</sup> 1/2**
    /// and **M<sup>+</sup> Code**.
    pub cap_height: f32,
    /// The top of the small letter _x_, defined as _520/1000 em_-size for both **M<sup>+</sup>
    /// 1/2** and **M<sup>+</sup> Code**.
    pub x_height: f32,
    /// The baseline.
    pub baseline: f32,
    /// Typographic descender, defined as _-120/1000 em_-size for **M<sup>+</sup> 1/2** and
    /// _-235/1000 em_-size for **M<sup>+</sup> Code**.
    pub descender: f32,
    /// The bottom of the line, defined as _-288/1000 em_-size for **M<sup>+</sup> 1/2** and
    /// _-270/1000 em_-size for **M<sup>+</sup> Code**.
    pub bottom: f32,
}

/// Decoration dimensions for [`mplusfonts`](../mplusfonts/index.html).
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DecorationDimensions {
    /// The offset from the baseline to the top of the decoration, defined as either _-100/1000
    /// em_-size in the case of [`underline`] or _312/1000 em_-size in the case of [`strikethrough`]
    /// for both **M<sup>+</sup> 1/2** and **M<sup>+</sup> Code**.
    ///
    /// [`underline`]: ../mplusfonts/struct.BitmapFont.html#structfield.underline
    /// [`strikethrough`]: ../mplusfonts/struct.BitmapFont.html#structfield.strikethrough
    pub offset: f32,
    /// The height of the decoration, defined as _50/1000 em_-size for both **M<sup>+</sup> 1/2**
    /// and **M<sup>+</sup> Code**.
    pub height: f32,
}

impl BitmapFontMetrics {
    /// Metrics of the invisible bitmap font.
    pub const NULL: Self = Self {
        top: 0.0,
        ascender: 0.0,
        cap_height: 0.0,
        x_height: 0.0,
        baseline: 0.0,
        descender: 0.0,
        bottom: 0.0,
    };

    /// Returns the _y_-offset for the specified text baseline in pixels.
    pub const fn y_offset(&self, baseline: Baseline) -> i32 {
        match baseline {
            Baseline::Top => self.top as i32 + (self.top % 1.0 > 0.0) as i32,
            Baseline::Bottom => self.bottom as i32 + (self.bottom % 1.0 > 0.0) as i32,
            Baseline::Middle => (self.x_height / 2.0) as i32 + (self.x_height % 2.0 > 0.0) as i32,
            Baseline::Alphabetic => self.baseline as i32 + (self.baseline % 1.0 > 0.0) as i32,
        }
    }

    /// Returns the line height in pixels.
    pub const fn line_height(&self) -> u32 {
        let height = self.y_offset(Baseline::Top) - self.y_offset(Baseline::Bottom);

        height as u32
    }
}

impl DecorationDimensions {
    /// The dimensions of the invisible decoration.
    pub const NULL: Self = Self {
        offset: 0.0,
        height: 0.0,
    };

    /// Returns the _y_-offset for a line primitive in pixels.
    pub const fn y_offset(&self) -> i32 {
        self.offset as i32 + (self.offset % 1.0 > 0.0) as i32
    }

    /// Returns the stroke width for a line primitive in pixels.
    pub const fn stroke_width(&self) -> u32 {
        let height = self.height as i32 + (self.height % 1.0 > 0.0) as i32;

        height as u32
    }
}
