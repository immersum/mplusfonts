use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::sync::RwLock;

use crate::mplus::bitmap::{Glyph, GlyphList};
use crate::mplus::charmap::CharmapEntry;

#[derive(Clone, Copy)]
pub struct CharDictionary<'a>(&'a RwLock<BTreeMap<String, CharmapEntry>>);

impl<'a> CharDictionary<'a> {
    pub fn new(entries: &'a RwLock<BTreeMap<String, CharmapEntry>>) -> Self {
        Self(entries)
    }

    pub fn contains_key(self, entry_key: &String) -> bool {
        let Self(entries) = self;
        let entries = entries.read().expect("expected no-poison lock on entries");

        entries.contains_key(entry_key)
    }

    pub fn insert_glyphs(self, entry_key: String, glyphs: Vec<Glyph>) {
        let Self(entries) = self;
        let mut entries = entries.write().expect("expected no-poison lock on entries");
        let entry = entries.entry(entry_key);
        entry.or_insert_with_key(|key| CharmapEntry {
            key: key.clone(),
            advance_chars: key.chars().count(),
            advance_width_to: BTreeMap::new(),
            advance_width: glyphs.iter().map(|glyph| glyph.advance_width).sum(),
            glyphs: GlyphList(glyphs),
        });
    }

    pub fn insert_advance_width(self, entry_key: String, to_entry_key: String, advance_width: f32) {
        let Self(entries) = self;
        let mut entries = entries.write().expect("expected no-poison lock on entries");
        let entry = entries.entry(entry_key);
        let entry = entry.and_modify(|entry| {
            if entry.advance_width != advance_width {
                if let Some(previous) = entry.advance_width_to.insert(to_entry_key, advance_width) {
                    debug_assert_eq!(
                        previous,
                        advance_width,
                        "expected no change in value in case of update to entry with key `{key:?}`",
                        key = entry.key,
                    );
                }
            }
        });
        debug_assert!(
            matches!(entry, Entry::Occupied(_)),
            "expected to modify an existing entry"
        );
    }
}
