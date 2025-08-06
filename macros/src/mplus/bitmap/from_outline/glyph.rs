use std::collections::BTreeMap;
use std::sync::Mutex;
use std::thread;

use swash::GlyphMetrics;
use swash::scale::{Render, Scaler, Source};
use swash::zeno::Vector;

use crate::mplus::bitmap::color;
use crate::mplus::bitmap::{Glyph, Image, ImageList};

use super::PixelAlignmentStrategy;

pub fn scale(
    scalers: &mut [Scaler],
    is_code: bool,
    pixel_alignment_strategy: PixelAlignmentStrategy,
    positions: u8,
    bit_depth: u8,
    glyph_metrics: &GlyphMetrics,
    glyph_offsets: (u16, f32, f32),
) -> Glyph {
    debug_assert_eq!(scalers.len(), positions as usize);
    let (id, mut x_offset, mut y_offset) = glyph_offsets;
    let mut advance_width = glyph_metrics.advance_width(id);
    let mut advance_height = glyph_metrics.advance_height(id);
    let advance_width_adjustment;
    if is_code {
        match pixel_alignment_strategy {
            PixelAlignmentStrategy::Floor(halfwidth) => {
                advance_width = advance_width.floor();
                advance_width_adjustment = halfwidth.adjustment(advance_width);
                advance_height = advance_height.floor();
                x_offset = x_offset.ceil();
            }
            PixelAlignmentStrategy::Ceil => {
                advance_width = (advance_width / 1.2).ceil();
                advance_width_adjustment = 0.0;
                advance_height = advance_height.ceil();
                x_offset = x_offset.ceil();
            }
            PixelAlignmentStrategy::Zero => {
                advance_width = 0.0;
                advance_width_adjustment = 0.0;
                advance_height = 0.0;
                x_offset = 0.0;
            }
        }
    } else {
        advance_width_adjustment = 0.0;
    }

    if [812, 813, 814, 815, 817, 818, 819, 820, 821, 823, 825].contains(&id) {
        x_offset = 0.0;
        y_offset = 0.0;
    }

    let is_square = advance_width == advance_height || id == 0;
    let positions = if is_code || is_square { 1 } else { positions };
    let x_padding = (advance_width_adjustment / 2.0) as u32;
    advance_width += advance_width_adjustment;

    let images = Mutex::new(BTreeMap::new());
    if advance_width > 0.0 {
        thread::scope(|scope| {
            let images = &images;
            (0..positions).zip(scalers).for_each(|(index, scaler)| {
                scope.spawn(move || {
                    let x_offset = f32::from(index) / f32::from(positions);
                    let image = Render::new(&[Source::Outline])
                        .offset(Vector::new(x_offset, 0.0))
                        .render(scaler, id)
                        .expect("expected glyph outline");

                    if image.data.is_empty() {
                        return;
                    }

                    let image = Image {
                        left: image.placement.left.saturating_add_unsigned(x_padding),
                        top: image.placement.top,
                        width: image.placement.width,
                        data: color::quantize(&image.data, image.placement.width, bit_depth),
                    };

                    let mut images = images.lock().expect("expected no-poison lock on images");
                    let image = images.insert(index, image);
                    debug_assert!(image.is_none(), "expected not to remove an existing image");
                });
            });
        });
    }

    let images = images
        .into_inner()
        .expect("expected no-poison lock on images");

    let images: Vec<_> = images.into_values().collect();
    if !images.is_empty() {
        debug_assert_eq!(images.len(), positions as usize);
    }

    Glyph {
        x_offset,
        y_offset,
        positions,
        bit_depth,
        id,
        advance_width,
        images: ImageList(images),
    }
}
