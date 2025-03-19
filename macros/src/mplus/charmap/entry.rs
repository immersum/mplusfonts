use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use crate::mplus::bitmap::GlyphList;

pub struct CharmapEntry {
    pub key: String,
    pub advance_chars: usize,
    pub advance_width_to: BTreeMap<String, f32>,
    pub advance_width: f32,
    pub glyphs: GlyphList,
}

impl ToTokens for CharmapEntry {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let CharmapEntry {
            key,
            advance_chars,
            advance_width_to: map,
            advance_width: default,
            glyphs,
        } = self;

        let map = map.iter().map(|(key, value)| quote!(#key => #value));
        let advance_width_to = quote! {
            |key| match key {
                #(#map,)*
                _ => #default,
            }
        };
        let entry = quote! {
            ::mplusfonts::CharmapEntry {
                key: #key,
                advance_chars: #advance_chars,
                advance_width_to: #advance_width_to,
                glyph: #glyphs,
            }
        };

        tokens.extend(entry);
    }
}
