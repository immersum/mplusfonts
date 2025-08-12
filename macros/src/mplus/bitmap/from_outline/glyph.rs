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
        let Self(id, x_offset, y_offset) = *self;
        debug_assert_eq!(scalers.len(), positions as usize);

        let advance_width = glyph_metrics.advance_width(id);
        let advance_height = glyph_metrics.advance_height(id);
        let new_advance_width = glyph_aligner.round_halfwidths(advance_width);
        let x_offset = glyph_aligner.round_counteract(x_offset);
        let x_padding = ((new_advance_width - advance_width) / 2.0) as u32;
        let is_square = advance_width == advance_height || id == 0;
        let advance_width = new_advance_width;

        let GlyphAligner { is_code, ref dir } = *glyph_aligner;
        let images = if let GlyphAlignDir::Unit = dir {
            Vec::new()
        } else {
            let positions = if is_code || is_square { 1 } else { positions };
            let images = Mutex::new(BTreeMap::new());
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

            let images = images
                .into_inner()
                .expect("expected no-poison lock on images");

            let images: Vec<_> = images.into_values().collect();
            debug_assert!(
                images.is_empty() || images.len() == positions as usize,
                "expected either no images at all or one image for each sub-pixel offset"
            );

            images
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
