mod entry;

use std::collections::BTreeMap;

pub use entry::CharmapEntry;

pub struct Charmap(pub Option<CharmapEntry>, pub BTreeMap<char, Charmap>);

impl FromIterator<(String, CharmapEntry)> for Charmap {
    fn from_iter<T: IntoIterator<Item = (String, CharmapEntry)>>(entries: T) -> Self {
        let mut payload = None;
        let mut groups = BTreeMap::new();
        for (mut key, entry) in entries {
            let duplicate = if key.is_empty() {
                payload.replace(entry)
            } else {
                let first = key.remove(0);
                groups
                    .entry(first)
                    .or_insert_with(BTreeMap::new)
                    .insert(key, entry)
            };
            let duplicate = duplicate.map(|entry| entry.key);
            debug_assert_eq!(None, duplicate, "expected field `0` to have a unique value");
        }

        let mut charmap = BTreeMap::new();
        for (key, group) in groups {
            let charmap = charmap.insert(key, Self::from_iter(group));
            debug_assert!(charmap.is_none(), "expected unique key: `{key:?}`");
        }

        Charmap(payload, charmap)
    }
}
