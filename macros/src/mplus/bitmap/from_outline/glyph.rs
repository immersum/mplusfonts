mod aligner;
mod offsets;

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::thread;

use swash::GlyphMetrics;
use swash::scale::{Render, Scaler, Source};
use swash::zeno::Vector;

use crate::mplus::bitmap::color;
use crate::mplus::bitmap::{Glyph, Image, ImageList};

pub use aligner::{GlyphAlignDir, GlyphAligner};
pub use offsets::GlyphOffsets;

impl GlyphOffsets {
    pub fn scale(
        &self,
        scalers: &mut [Scaler],
        positions: u8,
        bit_depth: u8,
        glyph_metrics: &GlyphMetrics,
        glyph_aligner: &GlyphAligner,
    ) -> Glyph {
        debug_assert_eq!(scalers.len(), positions as usize);
        let (is_initial, id, x_offset, y_offset) = match *self {
            Self::Initial(id, x_offset, y_offset) => (true, id, x_offset, y_offset),
            Self::Overlay(id, x_offset, y_offset) => (false, id, x_offset, y_offset),
        };

        let advance_width = glyph_metrics.advance_width(id);
        let advance_height = glyph_metrics.advance_height(id);
        let new_advance_width = glyph_aligner.round_halfwidths(advance_width);
        let x_offset = glyph_aligner.round_counteract(x_offset);
        let x_padding = ((new_advance_width - advance_width) / 2.0) as u32;
        let is_repeated = glyph_aligner.is_code || advance_width == advance_height || id == 0;
        let advance_width = new_advance_width;

        let images = if let GlyphAlignDir::Floor(_) | GlyphAlignDir::Ceil = glyph_aligner.dir {
            if is_initial || x_offset < 0.0 || advance_width > 0.0 {
                let count = if is_repeated { 1 } else { positions };
                let images = Mutex::new(BTreeMap::new());
                thread::scope(|scope| {
                    let images = &images;
                    (0..count).zip(scalers).for_each(|(index, scaler)| {
                        scope.spawn(move || {
                            let x_offset = f32::from(index) / f32::from(count);
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
                                data: color::quantize(
                                    &image.data,
                                    image.placement.width,
                                    bit_depth,
                                ),
                            };

                            images
                                .lock()
                                .expect("expected no-poison lock on images")
                                .insert(index, image);
                        });
                    });
                });

                images
                    .into_inner()
                    .expect("expected no-poison lock on images")
                    .into_values()
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

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
}
