//! Styles and style builders.
//!
//! Styles provide the settings for the text and background colors, and it also implements the
//! interface for rendering [text in `embedded-graphics`](embedded_graphics::text). A style builder
//! with a fluent-style interface is also available. Note that [`mplus!`](mplusfonts_macros::mplus)
//! provides anti-aliased bitmap fonts (using color types: [`Gray2`], [`Gray4`], and [`Gray8`]) ---
//! so, there is a trade-off:
//!
//! <div class="warning">
//!   This crate does not support background transparency. If no background color is specified, it
//!   defaults to black; this color is filled in from top to bottom, for the length of the text run.
//! </div>

use core::iter;

use embedded_graphics::Drawable;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, Point, Size};
use embedded_graphics::iterator::raw::RawDataSlice;
use embedded_graphics::pixelcolor::raw::BigEndian;
use embedded_graphics::pixelcolor::{BinaryColor, Gray2, Gray4, Gray8, PixelColor};
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use embedded_graphics::text::renderer::{CharacterStyle, TextMetrics, TextRenderer};
use embedded_graphics::text::{Baseline, DecorationColor};

use crate::adapter::DrawTargetExt;
use crate::charmap::{Charmap, CharmapEntry};
use crate::color::{Colormap, Invert, Linear, Screen};
use crate::font::BitmapFont;
use crate::glyph::NextGlyph;
use crate::image::{Image, ImageRaw, Mixed};
use crate::rect::RectangleExt;

pub use crate::builder::BitmapFontStyleBuilder;

/// Style using a bitmap font.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BitmapFontStyle<'a, 'b, T, C, const N: usize>
where
    C: PixelColor + From<C::Raw>,
    T: PixelColor + Default + Invert + Screen,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    /// The bitmap font.
    pub font: &'b BitmapFont<'a, C, N>,
    /// The text color.
    pub text_color: Option<T>,
    /// The background color.
    pub background_color: Option<T>,
    /// The underline color.
    pub underline_color: DecorationColor<T>,
    /// The strikethrough color.
    pub strikethrough_color: DecorationColor<T>,
}

impl<'a, 'b, T, C, const N: usize> BitmapFontStyle<'a, 'b, T, C, N>
where
    C: PixelColor + From<C::Raw>,
    T: PixelColor + Default + Invert + Screen,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    /// Creates a new style with the specified bitmap font and text color.
    pub const fn new(font: &'b BitmapFont<'a, C, N>, text_color: T) -> Self {
        BitmapFontStyleBuilder::<'_, '_, _, BinaryColor, 0>::new()
            .text_color(text_color)
            .font(font)
            .build()
    }

    /// Returns the text color, falling back to the inverse of the default value for type `T` when
    /// not set to a value.
    fn text_color(&self) -> T {
        self.text_color.unwrap_or(T::default().invert())
    }

    /// Returns the background_color, falling back to the default value for type `T` when not set
    /// to a value.
    fn background_color(&self) -> T {
        self.background_color.unwrap_or_default()
    }

    /// Returns the optional underline color, which, when set to a value, can either have the same
    /// color as the text or a custom color.
    fn underline_color(&self) -> Option<T> {
        match self.underline_color {
            DecorationColor::None => None,
            DecorationColor::TextColor => Some(self.text_color()),
            DecorationColor::Custom(color) => Some(color),
        }
    }

    /// Returns the optional strikethrough color, which, when set to a value, can either have the
    /// same color as the text or a custom color.
    fn strikethrough_color(&self) -> Option<T> {
        match self.strikethrough_color {
            DecorationColor::None => None,
            DecorationColor::TextColor => Some(self.text_color()),
            DecorationColor::Custom(color) => Some(color),
        }
    }
}

impl<'a, T, C, const N: usize> CharacterStyle for BitmapFontStyle<'a, '_, T, C, N>
where
    C: PixelColor + From<C::Raw>,
    T: PixelColor + Default + Invert + Screen,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    type Color = T;
}

macro_rules! impl_text_renderer {
    (
        $(
            $color_type:ty, $array_length:literal,
        )*
    ) => {
        $(
            impl<T, const N: usize> TextRenderer for BitmapFontStyle<'_, '_, T, $color_type, N>
            where
                T: PixelColor + Default + Invert + Screen,
                Colormap<T, $array_length>: Linear<T>,
            {
                type Color = T;

                fn draw_string<D>(
                    &self,
                    text: &str,
                    position: Point,
                    baseline: Baseline,
                    target: &mut D,
                ) -> Result<Point, D::Error>
                where
                    D: DrawTarget<Color = Self::Color>,
                {
                    let mut right = position.x;
                    let mut x = position.x as f32;
                    let y = position.y.saturating_add(self.font.metrics.y_offset(baseline));
                    let top = y.saturating_sub(self.font.metrics.y_offset(Baseline::Top));
                    let bottom = y.saturating_sub(self.font.metrics.y_offset(Baseline::Bottom));
                    let height = bottom.saturating_sub(top).try_into().unwrap_or_default();
                    let background_style = PrimitiveStyle::with_fill(self.background_color());
                    let line_strip = Rectangle {
                        top_left: Point::new(position.x, top),
                        size: Size::new(u32::MAX, height),
                    };

                    let colormap = Colormap::linear(self.background_color(), self.text_color());
                    let images = images_of_chars(&self.font.charmap, text, &mut x, y as f32);
                    let mut previous_line_pieces: Option<[Rectangle; 2]> = None;
                    let mut previous_image: Option<Image<_>> = None;
                    let mut right_of_overlay: Option<i32> = None;
                    for (image, is_overlay) in images {
                        let image_box = image.bounding_box();
                        let x = image_box.top_left.x.saturating_add_unsigned(image_box.size.width);
                        let clip_area = if let Some(previous_image) = previous_image.as_ref() {
                            let previous_image_box = previous_image.bounding_box();
                            let clip_area = previous_image_box.right_half();
                            if is_overlay {
                                for quadrant in previous_line_pieces.take().iter().flatten() {
                                    quadrant.draw_styled(&background_style, target)?;
                                }

                                let clip_area = image_box.y_reduce(top, bottom);
                                let above = clip_area.above(&previous_image_box);
                                let below = clip_area.below(&previous_image_box);
                                let mut adapter = target.value_mapped(&colormap);
                                image.clipped(&above).draw(&mut adapter)?;
                                image.clipped(&below).draw(&mut adapter)?;
                                image.mixed(previous_image, &colormap).draw(target)?;

                                let width = x.saturating_sub(right).try_into().unwrap_or_default();
                                let line_piece = Rectangle {
                                    top_left: Point::new(right, top),
                                    size: Size::new(width, height),
                                };

                                let xb_quadrant = line_piece.above(&image_box);
                                let xp_quadrant = line_piece.below(&image_box);
                                previous_line_pieces.replace([xb_quadrant, xp_quadrant]);

                                if x > right {
                                    right = x;
                                }

                                right_of_overlay.replace(right);
                                continue;
                            }

                            if let Some([xb_quadrant, xp_quadrant]) = previous_line_pieces.take() {
                                let mut adapter = target.value_mapped(&colormap);
                                image.clipped(&xb_quadrant).draw(&mut adapter)?;
                                image.clipped(&xp_quadrant).draw(&mut adapter)?;

                                let xb = xb_quadrant.left_of(&image_box);
                                let xp = xp_quadrant.left_of(&image_box);
                                xb.draw_styled(&background_style, target)?;
                                xp.draw_styled(&background_style, target)?;
                            }

                            let clip_area = clip_area.left_of(&image_box);
                            let mut adapter = target.value_mapped(&colormap);
                            previous_image.clipped(&clip_area).draw(&mut adapter)?;

                            let line_piece = previous_image_box.y_extend(top, bottom);
                            let line_piece = line_piece.right_of(&clip_area);
                            let dx_quadrant = line_piece.above(&image_box);
                            let qx_quadrant = line_piece.below(&image_box);
                            previous_image.clipped(&dx_quadrant).draw(&mut adapter)?;
                            previous_image.clipped(&qx_quadrant).draw(&mut adapter)?;

                            let dx = dx_quadrant.indent_to(right_of_overlay.unwrap_or_default());
                            let qx = qx_quadrant.indent_to(right_of_overlay.unwrap_or_default());
                            dx.above(&previous_image_box).draw_styled(&background_style, target)?;
                            qx.above(&previous_image_box).draw_styled(&background_style, target)?;
                            dx.below(&previous_image_box).draw_styled(&background_style, target)?;
                            qx.below(&previous_image_box).draw_styled(&background_style, target)?;

                            image.mixed(previous_image, &colormap).draw(target)?;

                            let line_piece = line_strip.left_of(&image_box);
                            let line_piece = line_piece.indent_to(right);
                            line_piece.draw_styled(&background_style, target)?;

                            image_box.left_half().right_of(&previous_image_box)
                        } else {
                            let line_piece = line_strip.left_of(&image_box);
                            line_piece.draw_styled(&background_style, target)?;

                            image_box.left_half()
                        };

                        let line_piece = clip_area.y_extend(top, bottom);
                        let line_piece = line_piece.indent_to(right);
                        line_piece.above(&image_box).draw_styled(&background_style, target)?;
                        line_piece.below(&image_box).draw_styled(&background_style, target)?;

                        let line_piece = image_box.y_extend(top, bottom);
                        let line_piece = line_piece.right_of(&clip_area);
                        let xb_quadrant = line_piece.above(&image_box);
                        let xp_quadrant = line_piece.below(&image_box);
                        previous_line_pieces.replace([xb_quadrant, xp_quadrant]);

                        let mut adapter = target.value_mapped(&colormap);
                        image.clipped(&clip_area).draw(&mut adapter)?;
                        previous_image.replace(image);

                        if x > right {
                            right = x;
                        }
                    }

                    if let Some(previous_image) = previous_image.take() {
                        for quadrant in previous_line_pieces.take().iter().flatten() {
                            quadrant.draw_styled(&background_style, target)?;
                        }

                        let previous_image_box = previous_image.bounding_box();
                        let clip_area = previous_image_box.right_half();
                        let mut adapter = target.value_mapped(&colormap);
                        previous_image.clipped(&clip_area).draw(&mut adapter)?;
                    }

                    let width = (x as i32).saturating_sub(right).try_into().unwrap_or_default();
                    let line_piece = Rectangle {
                        top_left: Point::new(right, top),
                        size: Size::new(width, height),
                    };

                    let next_position = Point::new(x as i32, position.y);
                    line_piece.draw_styled(&background_style, target)?;

                    let right = i32::max(x as i32, right);
                    let width = right.saturating_sub(position.x);
                    let width = width.try_into().unwrap_or_default();

                    if let Some(stroke_color) = self.underline_color() {
                        let top = y.saturating_sub(self.font.underline.y_offset());
                        let height = self.font.underline.stroke_width();
                        let underline_style = PrimitiveStyle::with_fill(stroke_color);
                        let underline = Rectangle {
                            top_left: Point::new(position.x, top),
                            size: Size::new(width, height),
                        };

                        underline.draw_styled(&underline_style, target)?;
                    }

                    if let Some(stroke_color) = self.strikethrough_color() {
                        let top = y.saturating_sub(self.font.strikethrough.y_offset());
                        let height = self.font.strikethrough.stroke_width();
                        let strikethrough_style = PrimitiveStyle::with_fill(stroke_color);
                        let strikethrough = Rectangle {
                            top_left: Point::new(position.x, top),
                            size: Size::new(width, height),
                        };

                        strikethrough.draw_styled(&strikethrough_style, target)?;
                    }

                    Ok(next_position)
                }

                fn draw_whitespace<D>(
                    &self,
                    width: u32,
                    position: Point,
                    baseline: Baseline,
                    target: &mut D,
                ) -> Result<Point, D::Error>
                where
                    D: DrawTarget<Color = Self::Color>,
                {
                    let x = position.x as f32 + width as f32;
                    let y = position.y.saturating_add(self.font.metrics.y_offset(baseline));
                    let top = y.saturating_sub(self.font.metrics.y_offset(Baseline::Top));
                    let bottom = y.saturating_sub(self.font.metrics.y_offset(Baseline::Bottom));
                    let height = bottom.saturating_sub(top).try_into().unwrap_or_default();
                    let background_style = PrimitiveStyle::with_fill(self.background_color());
                    let line_piece = Rectangle {
                        top_left: Point::new(position.x, top),
                        size: Size::new(width, height),
                    };

                    let next_position = Point::new(x as i32, position.y);
                    line_piece.draw_styled(&background_style, target)?;

                    if let Some(stroke_color) = self.underline_color() {
                        let top = y.saturating_sub(self.font.underline.y_offset());
                        let height = self.font.underline.stroke_width();
                        let underline_style = PrimitiveStyle::with_fill(stroke_color);
                        let underline = Rectangle {
                            top_left: Point::new(position.x, top),
                            size: Size::new(width, height),
                        };

                        underline.draw_styled(&underline_style, target)?;
                    }

                    if let Some(stroke_color) = self.strikethrough_color() {
                        let top = y.saturating_sub(self.font.strikethrough.y_offset());
                        let height = self.font.strikethrough.stroke_width();
                        let strikethrough_style = PrimitiveStyle::with_fill(stroke_color);
                        let strikethrough = Rectangle {
                            top_left: Point::new(position.x, top),
                            size: Size::new(width, height),
                        };

                        strikethrough.draw_styled(&strikethrough_style, target)?;
                    }

                    Ok(next_position)
                }

                fn measure_string(
                    &self,
                    text: &str,
                    position: Point,
                    baseline: Baseline
                ) -> TextMetrics {
                    let mut right = position.x;
                    let mut x = position.x as f32;
                    let y = position.y.saturating_add(self.font.metrics.y_offset(baseline));
                    let top = y.saturating_sub(self.font.metrics.y_offset(Baseline::Top));
                    let bottom = y.saturating_sub(self.font.metrics.y_offset(Baseline::Bottom));
                    let height = bottom.saturating_sub(top).try_into().unwrap_or_default();
                    let images = images_of_chars(&self.font.charmap, text, &mut x, y as f32);
                    for (image, _) in images {
                        let image_box = image.bounding_box();
                        let x = image_box.top_left.x.saturating_add_unsigned(image_box.size.width);
                        if x > right {
                            right = x;
                        }
                    }

                    let next_position = Point::new(x as i32, position.y);
                    let width = right.saturating_sub(position.x).try_into().unwrap_or_default();
                    let bounding_box = Rectangle {
                        top_left: Point::new(position.x, top),
                        size: Size::new(width, height),
                    };

                    TextMetrics { bounding_box, next_position }
                }

                fn line_height(&self) -> u32 {
                    self.font.metrics.line_height()
                }
            }
        )*
    }
}

impl_text_renderer! {
    BinaryColor, 2,
    Gray2, 4,
    Gray4, 16,
    Gray8, 256,
}

fn images_of_chars<'a, C, const N: usize>(
    charmap: &Charmap<'a, C, N>,
    text: &str,
    x: &mut f32,
    y: f32,
) -> impl IntoIterator<Item = (Image<ImageRaw<'a, C>>, bool)>
where
    C: PixelColor + From<C::Raw>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    let mut chars = text.chars();
    let mut next_glyph = None;
    let mut next_entry = None;
    let mut previous_entry = None;
    iter::from_fn(move || {
        let entry = match next_entry {
            Some(entry) => entry,
            None => {
                let slice = chars.as_str();
                if slice.is_empty() {
                    *x += previous_entry
                        .take()
                        .map(|entry: &CharmapEntry<C, N>| entry.advance_width_to)
                        .map(|advance_width_to| advance_width_to(Default::default()))
                        .unwrap_or_default();

                    return next_glyph.map(|next: &NextGlyph<C, N>| {
                        let x = *x + next.x_offset;
                        let y = y - next.y_offset;
                        let image = next.glyph.images.get((x * N as f32) as usize);
                        let image = image.mul_offset(1, -1).add_offset(x as i32, y as i32);
                        next_glyph = next.glyph.next;

                        (image, true)
                    });
                }

                let entry = charmap.get(slice);
                *x += previous_entry
                    .replace(entry)
                    .map(|entry| entry.advance_width_to)
                    .map(|advance_width_to| advance_width_to(entry.key))
                    .unwrap_or_default();

                for _ in 0..entry.advance_chars {
                    let _ = chars.next();
                }

                entry
            }
        };

        let tuple = match next_glyph {
            Some(next) => {
                let x = *x + next.x_offset;
                let y = y - next.y_offset;
                let image = next.glyph.images.get((x * N as f32) as usize);
                let image = image.mul_offset(1, -1).add_offset(x as i32, y as i32);
                next_glyph = next.glyph.next;
                next_entry = Some(entry);

                (image, true)
            }
            None => {
                let image = entry.glyph.images.get((*x * N as f32) as usize);
                let image = image.mul_offset(1, -1).add_offset(*x as i32, y as i32);
                next_glyph = entry.glyph.next;
                next_entry = None;

                (image, false)
            }
        };

        Some(tuple)
    })
}
