mod offsets;
mod spacing;

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::thread;

use swash::GlyphMetrics;
use swash::scale::{Render, Scaler, Source};
use swash::zeno::Vector;

use crate::mplus::bitmap::color;
use crate::mplus::bitmap::units::Halfwidth;
use crate::mplus::bitmap::{Glyph, Image, ImageList};

pub use offsets::GlyphOffsets;
pub use spacing::GlyphSpacing;

impl GlyphOffsets {
    pub fn scale(
        &self,
        scalers: &mut [Scaler],
        positions: u8,
        bit_depth: u8,
        glyph_metrics: &GlyphMetrics,
        glyph_spacing: &GlyphSpacing,
    ) -> Glyph {
        debug_assert_eq!(scalers.len(), positions as usize);
        let advance_width = glyph_metrics.advance_width(self.id);
        let advance_height = glyph_metrics.advance_height(self.id);
        let new_advance_width = glyph_spacing.halfwidths(advance_width);
        let centering_offset = (new_advance_width - advance_width) / 2.0;
        let x_offset = glyph_spacing.compensate(self.x_offset);
        let y_offset = self.y_offset;
        let is_repeating = glyph_spacing.is_code || advance_width == advance_height || self.id == 0;
        let images = if let Halfwidth::Floor(_) | Halfwidth::Ceil = glyph_spacing.halfwidth {
            if !self.is_overlay || self.x_offset < 0.0 {
                let length = if is_repeating { 1 } else { positions };
                let images = Mutex::new(BTreeMap::new());
                thread::scope(|scope| {
                    let images = &images;
                    (0..length).zip(scalers).for_each(|(index, scaler)| {
                        let x_offset = x_offset.fract() + f32::from(index) / f32::from(length);
                        let y_offset = y_offset.fract();
                        scope.spawn(move || {
                            let image = Render::new(&[Source::Outline])
                                .offset(Vector::new(x_offset, y_offset))
                                .render(scaler, self.id)
                                .expect("expected glyph outline");

                            if image.data.is_empty() {
                                return;
                            }

                            let left = image.placement.left;
                            let left = left.saturating_add_unsigned(centering_offset as u32);
                            let top = image.placement.top;
                            let width = image.placement.width;
                            let data = color::quantize(&image.data, width, bit_depth);
                            let image = Image {
                                left,
                                top,
                                width,
                                data,
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
            x_offset: x_offset - x_offset.fract(),
            y_offset: y_offset - y_offset.fract(),
            positions,
            bit_depth,
            id: self.id,
            advance_width: new_advance_width,
            images: ImageList(images),
        }
    }
}
