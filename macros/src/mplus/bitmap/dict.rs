use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::sync::RwLock;

use crate::mplus::bitmap::{Glyph, GlyphList};
use crate::mplus::charmap::CharmapEntry;

#[derive(Clone, Copy)]
pub struct CharDictionary<'a>(&'a RwLock<BTreeMap<String, CharmapEntry>>);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CharDictionaryKey(String);

impl<'a> CharDictionary<'a> {
    pub fn new(entries: &'a RwLock<BTreeMap<String, CharmapEntry>>) -> Self {
        Self(entries)
    }

    pub fn contains_key(&self, entry_key: &CharDictionaryKey) -> bool {
        let Self(entries) = self;
        let entries = entries.read().expect("expected no-poison lock on entries");
        let key = entry_key.as_ref();

        entries.contains_key(key)
    }

    pub fn insert_glyphs(&self, entry_key: CharDictionaryKey, glyphs: Vec<Glyph>) {
        let Self(entries) = self;
        let mut entries = entries.write().expect("expected no-poison lock on entries");
        let key = entry_key.into_inner();
        let entry = entries.entry(key);
        entry.or_insert_with_key(|key| CharmapEntry {
            key: key.clone(),
            advance_chars: key.chars().count(),
            advance_width_to: BTreeMap::new(),
            advance_width: glyphs.iter().map(|glyph| glyph.advance_width).sum(),
            glyphs: GlyphList(glyphs),
        });
    }

    pub fn insert_advance_width(
        &self,
        entry_key: CharDictionaryKey,
        to_entry_key: CharDictionaryKey,
        advance_width: f32,
    ) {
        let Self(entries) = self;
        let mut entries = entries.write().expect("expected no-poison lock on entries");
        let key = entry_key.into_inner();
        let entry = entries.entry(key);
        let entry = entry.and_modify(|entry| {
            if entry.advance_width != advance_width {
                let to_key = to_entry_key.into_inner();
                if let Some(previous) = entry.advance_width_to.insert(to_key, advance_width) {
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

impl CharDictionaryKey {
    pub fn try_from(string: &str, start: usize, end: usize) -> Result<Self, ()> {
        let bytes: Vec<_> = string.bytes().skip(start).take(end - start).collect();
        debug_assert!(
            !bytes.is_empty(),
            "indexing into `{string:?}`, \
            out of bounds at `{end:?}`"
        );
        let key = match String::from_utf8(bytes) {
            Ok(substring) if substring.is_empty() => return Err(()),
            Ok(substring) => substring,
            Err(e) => {
                debug_assert_eq!(
                    None,
                    Some(e),
                    "indexing into `{string:?}`, \
                    expected character boundary at bytes `{start:?}` and `{end:?}`"
                );
                return Err(());
            }
        };

        Ok(Self(key))
    }

    pub fn into_inner(self) -> String {
        let Self(key) = self;

        key
    }
}

impl AsRef<str> for CharDictionaryKey {
    fn as_ref(&self) -> &str {
        let Self(key) = self;

        key
    }
}
