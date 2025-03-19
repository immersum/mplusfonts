use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use swash::GlyphId;

use crate::mplus::bitmap::ImageList;

pub struct Glyph {
    pub x_offset: f32,
    pub y_offset: f32,
    pub positions: u8,
    pub bit_depth: u8,
    pub id: GlyphId,
    pub advance_width: f32,
    pub images: ImageList,
}

pub struct GlyphList(pub Vec<Glyph>);

impl ToTokens for GlyphList {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self(vec) = self;
        let mut previous_glyph = None;
        for glyph in vec.iter().rev() {
            let Glyph {
                x_offset,
                y_offset,
                positions,
                bit_depth,
                id,
                advance_width: _,
                images,
            } = glyph;

            let next_glyph = match previous_glyph.take() {
                Some((x_offset, y_offset, glyph)) => {
                    let value = next_glyph(x_offset, y_offset, *positions, *bit_depth, glyph);

                    quote!(Some(#value))
                }
                None => quote!(None),
            };
            let glyph = quote! {
                ::mplusfonts::glyph::Glyph {
                    id: #id,
                    images: #images,
                    next: #next_glyph,
                }
            };

            let glyph = previous_glyph.replace((*x_offset, *y_offset, glyph));
            debug_assert!(glyph.is_none(), "expected token stream to be taken");
        }

        let glyph = match previous_glyph {
            Some((x_offset, y_offset, glyph)) => {
                debug_assert_eq!(0.0, x_offset, "expected glyph with no horizontal offset",);
                debug_assert_eq!(0.0, y_offset, "expected glyph with no vertical offset",);

                glyph
            }
            None => quote! {
                ::mplusfonts::glyph::Glyph::NULL
            },
        };

        tokens.extend(glyph);
    }
}

fn next_glyph(
    x_offset: f32,
    y_offset: f32,
    positions: u8,
    bit_depth: u8,
    glyph: impl ToTokens,
) -> impl ToTokens {
    let positions = positions as usize;
    let params = match bit_depth {
        1 => quote!(::embedded_graphics::pixelcolor::BinaryColor, #positions),
        2 => quote!(::embedded_graphics::pixelcolor::Gray2, #positions),
        4 => quote!(::embedded_graphics::pixelcolor::Gray4, #positions),
        8 => quote!(::embedded_graphics::pixelcolor::Gray8, #positions),
        x => panic!("expected one of: `1`, `2`, `4`, `8`; found: `{x}`"),
    };
    let next_glyph = quote! {
        ::mplusfonts::glyph::NextGlyph {
            x_offset: #x_offset,
            y_offset: #y_offset,
            glyph: #glyph,
        }
    };
    let value = quote!(const {
        const DATA: ::mplusfonts::glyph::NextGlyph<#params> = #next_glyph;

        &DATA
    });

    value
}
