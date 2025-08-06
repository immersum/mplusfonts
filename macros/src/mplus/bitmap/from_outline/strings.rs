use swash::shape::Shaper;
use swash::text::cluster::SourceRange;

use crate::mplus::bitmap::Glyph;

use super::{CharDictionary, PixelAlignmentStrategy};

pub fn shape_and_render(
    entries: CharDictionary,
    mut shaper: Shaper,
    mut render: impl FnMut((u16, f32, f32)) -> Glyph,
    is_fallback: bool,
    is_code: bool,
    pixel_alignment_strategy: PixelAlignmentStrategy,
    strings: Vec<&String>,
) {
    for string in strings.iter() {
        shaper.add_str(string);
        shaper.add_str("\n");
    }

    let mut strings = strings.into_iter();
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

        let Ok(entry_key) = try_to_entry_key(string, start as usize, end as usize) else {
            return;
        };
        let (glyphs, mut advance_width) = glyph_cluster
            .glyphs
            .iter()
            .filter(|glyph| glyph.id > 0 || entry_key == "\u{FFFD}")
            .fold((Vec::new(), 0.0), |(mut glyphs, advance_width), glyph| {
                let glyph_offsets = (glyph.id, glyph.x, glyph.y);
                let advance_width = advance_width + glyph.advance;
                glyphs.push(glyph_offsets);

                (glyphs, advance_width)
            });

        let advance_width_adjustment;
        if is_code {
            match pixel_alignment_strategy {
                PixelAlignmentStrategy::Floor(halfwidth) => {
                    advance_width = advance_width.floor();
                    advance_width_adjustment = halfwidth.adjustment(advance_width);
                }
                PixelAlignmentStrategy::Ceil => {
                    advance_width = (advance_width / 1.2).ceil();
                    advance_width_adjustment = 0.0;
                }
                PixelAlignmentStrategy::Zero => {
                    advance_width = 0.0;
                    advance_width_adjustment = 0.0;
                }
            }
        } else {
            advance_width_adjustment = 0.0;
        }

        advance_width += advance_width_adjustment;

        if !glyphs.is_empty() || glyph_cluster.is_empty() {
            if !entries.contains_key(&entry_key) {
                let glyphs = glyphs.into_iter().map(&mut render);
                entries.insert_glyphs(entry_key.clone(), glyphs.collect());
            }

            if is_fallback {
                return;
            }

            let to_entry_key = entry_key.clone();
            if let Some((entry_key, advance_width)) = previous.replace((entry_key, advance_width)) {
                entries.insert_advance_width(entry_key, to_entry_key, advance_width);
            }
        } else {
            previous = None;
        }
    });
}

fn try_to_entry_key(string: &str, start: usize, end: usize) -> Result<String, ()> {
    let bytes: Vec<_> = string.bytes().skip(start).take(end - start).collect();

    debug_assert!(
        !bytes.is_empty(),
        "indexing into `{string:?}`, out of bounds at `{end:?}`"
    );
    let entry_key = match String::from_utf8(bytes) {
        Ok(substring) if substring.is_empty() => return Err(()),
        Ok(substring) => substring,
        Err(e) => {
            let message = format!("expected character boundary at bytes `{start:?}` and `{end:?}`");
            debug_assert_eq!(None, Some(e), "indexing into `{string:?}`, {message}");
            return Err(());
        }
    };

    Ok(entry_key)
}
