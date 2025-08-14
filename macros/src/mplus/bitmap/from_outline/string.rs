use swash::shape::Shaper;
use swash::text::cluster::SourceRange;

use crate::mplus::bitmap::{CharDictionary, CharDictionaryKey, Glyph};

use super::glyph::{GlyphAligner, GlyphOffsets};

pub struct StringRefList<'a>(pub Vec<&'a String>);

impl StringRefList<'_> {
    pub fn shape_and_render(
        &self,
        entries: CharDictionary,
        mut shaper: Shaper,
        mut render: impl FnMut(GlyphOffsets) -> Glyph,
        glyph_aligner: &GlyphAligner,
        is_fallback: bool,
    ) {
        let Self(strings) = self;
        for string in strings.iter() {
            shaper.add_str(string);
            shaper.add_str("\n");
        }

        let mut strings = strings.iter();
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

            let Ok(entry_key) = CharDictionaryKey::try_from(string, start as usize, end as usize)
            else {
                return;
            };
            let glyphs = glyph_cluster
                .glyphs
                .iter()
                .filter(|glyph| glyph.id > 0 || entry_key.as_ref() == "\u{FFFD}")
                .peekable();

            let is_variable_width = glyph_aligner.is_code && !is_fallback;
            let mut advance_width = 0.0;
            let mut glyph_offsets = Vec::new();
            let mut is_overlay = false;
            for glyph in glyphs {
                advance_width += glyph.advance;
                glyph_offsets.push({
                    let mut glyph_offsets = GlyphOffsets::from_glyph(glyph, is_overlay);
                    glyph_offsets.patch(is_variable_width);

                    glyph_offsets
                });
                is_overlay = true;
            }

            let advance_width = glyph_aligner.round_halfwidths(advance_width);

            if !glyph_offsets.is_empty() || glyph_cluster.is_empty() {
                if !entries.contains_key(&entry_key) {
                    let glyphs = glyph_offsets.into_iter().map(&mut render).collect();
                    entries.insert_glyphs(entry_key.clone(), glyphs);
                }

                if is_fallback {
                    return;
                }

                let to_entry_key = entry_key.clone();
                if let Some(previous) = previous.replace((entry_key, advance_width)) {
                    let (entry_key, advance_width) = previous;
                    entries.insert_advance_width(entry_key, to_entry_key, advance_width);
                }
            } else {
                previous = None;
            }
        });
    }
}
