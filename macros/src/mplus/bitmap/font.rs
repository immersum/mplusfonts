use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use crate::mplus::charmap::{Charmap, CharmapEntry};

pub struct BitmapFont {
    pub charmap: Charmap,
    pub notdef: CharmapEntry,
    pub positions: u8,
    pub bit_depth: u8,
    pub size: f32,
    pub is_code: bool,
}

impl ToTokens for BitmapFont {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            charmap,
            notdef,
            positions,
            bit_depth,
            size,
            is_code,
        } = self;

        let positions = *positions as usize;
        let params = match bit_depth {
            1 => quote!(::embedded_graphics::pixelcolor::BinaryColor, #positions),
            2 => quote!(::embedded_graphics::pixelcolor::Gray2, #positions),
            4 => quote!(::embedded_graphics::pixelcolor::Gray4, #positions),
            8 => quote!(::embedded_graphics::pixelcolor::Gray8, #positions),
            x => panic!("expected one of: `1`, `2`, `4`, `8`; found: `{x}`"),
        };
        let charmap = charmap_tokens(charmap, notdef, &params);
        let metrics = metrics_tokens(*size, *is_code);
        let font = quote! {
            ::mplusfonts::BitmapFont::<#params> {
                charmap: #charmap,
                metrics: #metrics,
            }
        };

        tokens.extend(font);
    }
}

fn charmap_tokens(charmap: &Charmap, notdef: &CharmapEntry, params: &impl ToTokens) -> TokenStream {
    let Charmap(payload, charmap) = charmap;
    let payload = payload.as_ref().unwrap_or(notdef);
    let leaf = quote! {
        ::mplusfonts::Charmap::Leaf(#payload)
    };
    if charmap.is_empty() {
        return leaf;
    }

    let static_ref = |charmap| {
        quote!(const {
            const DATA: ::mplusfonts::Charmap<#params> = #charmap;

            &DATA
        })
    };
    let map = charmap.iter().map(|(key, charmap)| {
        let charmap = charmap_tokens(charmap, payload, params);
        let value = static_ref(charmap);
        quote!(#key => #value)
    });
    let default = static_ref(leaf);
    let branch = quote! {
        ::mplusfonts::Charmap::Branch(
            |key| match key {
                #(#map,)*
                _ => #default
            }
        )
    };

    branch
}

fn metrics_tokens(size: f32, is_code: bool) -> TokenStream {
    let top = size * if is_code { 1.235 } else { 1.16 };
    let ascender = size * if is_code { 1.0 } else { 0.86 };
    let cap_height = size * 0.73;
    let x_height = size * 0.52;
    let baseline = 0f32;
    let descender = size * if is_code { -0.235 } else { -0.12 };
    let bottom = size * if is_code { -0.27 } else { -0.288 };
    let metrics = quote! {
        ::mplusfonts::BitmapFontMetrics {
            top: #top,
            ascender: #ascender,
            cap_height: #cap_height,
            x_height: #x_height,
            baseline: #baseline,
            descender: #descender,
            bottom: #bottom,
        }
    };

    metrics
}
