mod color;
mod font;
mod glyph;
mod image;

use std::collections::BTreeMap;

use swash::scale::{Render, ScaleContext, Source};
use swash::shape::ShapeContext;
use swash::text::cluster::SourceRange;
use swash::zeno::Vector;

use super::Arguments;
use super::charmap::CharmapEntry;
use super::font::{Font, FontWidth};

pub use font::BitmapFont;
pub use glyph::{Glyph, GlyphList};
pub use image::{Image, ImageList};

pub fn render_glyphs(args: &Arguments, is_fallback: bool) -> BTreeMap<String, CharmapEntry> {
    let mut entries = BTreeMap::new();
    let font = args.font.value();
    let font_ref = font.as_ref(is_fallback);
    let is_code = matches!(font, Font::MPLUSCode { .. });

    let mut coords = Vec::new();
    let units = args.weight.into_value();
    let weight_axis = font_ref
        .variations()
        .find_by_tag(swash::tag_from_str_lossy("wght"))
        .expect("expected font weight axis");

    coords.push(weight_axis.normalize(units.into()));

    if is_fallback {
        if let Font::MPLUS1 { .. } | Font::MPLUS2 { .. } = font {
            return entries;
        }
    } else if let Font::MPLUSCode { variable, .. } = font {
        let (_, _, FontWidth(units)) = *variable;
        let width_axis = font_ref
            .variations()
            .find_by_tag(swash::tag_from_str_lossy("wdth"))
            .expect("expected font width axis");

        coords.push(width_axis.normalize(units.into()));
    }

    let pixels_per_em = args.size.into_value();
    let glyph_metrics = font_ref.glyph_metrics(&coords).scale(pixels_per_em);

    let mut context = ShapeContext::new();
    let mut shaper = context
        .builder(font_ref)
        .normalized_coords(&coords)
        .size(pixels_per_em)
        .features(&[("liga", !is_fallback as u16)])
        .build();

    let mut context = ScaleContext::new();
    let mut scaler = context
        .builder(font_ref)
        .normalized_coords(&coords)
        .size(pixels_per_em)
        .hint(args.hint.into_value())
        .build();

    let positions = args.positions.into_value();
    let bit_depth = args.bit_depth.into_value();
    let sources: Vec<_> = args
        .sources
        .iter()
        .flat_map(|source| source.strings(is_code))
        .collect();

    let mut strings = sources
        .iter()
        .flat_map(|source| source.iter())
        .filter(|string| !string.is_empty());

    for string in strings.clone() {
        shaper.add_str(string);
        shaper.add_str("\n");
    }

    let mut string = "";
    let mut newline = false;
    let mut previous = None;
    shaper.shape_with(|glyph_cluster| {
        let SourceRange { start, end } = glyph_cluster.source;
        if start == 0 {
            newline = !newline;
            if newline {
                string = strings.next().expect("expected string iterator to yield");
            } else {
                return;
            }
        }

        let bytes: Vec<_> = string
            .bytes()
            .skip(start as usize)
            .take(end as usize - start as usize)
            .collect();

        debug_assert_ne!(0, bytes.len(), r#"indexing into `"{string}"`, out of bounds at `{end}`"#);
        if bytes.is_empty() {
            return;
        }

        let entry_key = match String::from_utf8(bytes) {
            Ok(substring) => substring,
            Err(e) => {
                let message = format!("expected character boundary at bytes `{start}` and `{end}`");
                debug_assert_eq!(None, Some(e), r#"indexing into `"{string}"`, {message}"#);
                return;
            }
        };

        let entry_glyphs = glyph_cluster
            .glyphs
            .iter()
            .filter(|glyph| glyph.id > 0 || entry_key == "\u{FFFD}");

        if entry_glyphs.clone().count() > 0 || glyph_cluster.is_empty() {
            entries.entry(entry_key.clone()).or_insert_with(|| {
                let mut glyphs = Vec::new();
                for glyph in entry_glyphs.clone() {
                    let mut advance_width = glyph_metrics.advance_width(glyph.id);
                    let mut advance_height = glyph_metrics.advance_height(glyph.id);
                    if is_code {
                        advance_width = advance_width.floor();
                        advance_height = advance_height.floor();
                    }

                    let mut x_offset = if is_code { glyph.x.ceil() } else { glyph.x };
                    let mut y_offset = glyph.y;
                    if [812, 813, 814, 815, 817, 818, 819, 820, 821, 823, 825].contains(&glyph.id) {
                        x_offset = 0.0;
                        y_offset = 0.0;
                    }

                    let mut glyph = Glyph {
                        x_offset,
                        y_offset,
                        positions,
                        bit_depth,
                        id: glyph.id,
                        advance_width,
                        images: ImageList(Vec::new()),
                    };

                    let ImageList(ref mut images) = glyph.images;
                    let is_square = advance_width == advance_height;
                    let positions = if is_code || is_square { 1 } else { positions };
                    for index in 0..positions {
                        let x_offset = f32::from(index) / f32::from(positions);
                        let image = Render::new(&[Source::Outline])
                            .offset(Vector::new(x_offset, 0.0))
                            .render(&mut scaler, glyph.id)
                            .expect("expected glyph outline");

                        let image = Image {
                            left: image.placement.left,
                            top: image.placement.top,
                            width: image.placement.width,
                            data: color::quantize(&image.data, image.placement.width, bit_depth),
                        };

                        images.push(image);
                    }

                    glyphs.push(glyph);
                }

                CharmapEntry {
                    key: entry_key.clone(),
                    advance_chars: entry_key.chars().count(),
                    advance_width_to: BTreeMap::new(),
                    advance_width: glyphs.iter().map(|glyph| glyph.advance_width).sum(),
                    glyphs: GlyphList(glyphs),
                }
            });

            if is_fallback {
                return;
            }

            let mut entry_advance_width: f32 = entry_glyphs.map(|glyph| glyph.advance).sum();
            if is_code {
                entry_advance_width = entry_advance_width.floor();
            }

            let previous = previous.replace((entry_key.clone(), entry_advance_width));
            if let Some((previous_key, advance_width_to_entry)) = previous {
                let previous_entry = entries.get_mut(&previous_key).expect("expected entry");
                if previous_entry.advance_width != advance_width_to_entry {
                    let previous_advance_width_to_entry = previous_entry
                        .advance_width_to
                        .insert(entry_key, advance_width_to_entry);

                    if let Some(previous_advance_width_to_entry) = previous_advance_width_to_entry {
                        debug_assert_eq!(
                            previous_advance_width_to_entry, advance_width_to_entry,
                            r#"expected equal previous advance width for entry key `"{previous_key}"`"#
                        );
                    }
                }
            }
        } else {
            previous = None;
        }
    });

    entries
}
