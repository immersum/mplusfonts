use syn::parse::{Parse, ParseStream};
use syn::punctuated::{Pair, Punctuated};
use syn::{parse, Token};

use super::font::{Font, FontSize, FontWeight};
use super::CharSource;

pub struct Arguments {
    pub font: Pair<Font, Token![,]>,
    pub weight: Pair<u16, Token![,]>,
    pub size: Pair<f32, Token![,]>,
    pub hint: Pair<bool, Token![,]>,
    pub positions: Pair<u8, Token![,]>,
    pub bit_depth: Pair<u8, Token![,]>,
    pub sources: Punctuated<CharSource, Token![,]>,
}

impl Parse for Arguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let font = Pair::new(input.parse()?, input.parse()?);
        let weight = Pair::new(input.call(parse_weight(font.value()))?, input.parse()?);
        let size = Pair::new(input.call(parse_size)?, input.parse()?);
        let hint = Pair::new(input.call(parse_bool)?, input.parse()?);
        let positions = Pair::new(input.call(parse_u8_in_range::<1, 16>)?, input.parse()?);
        let bit_depth = input.call(parse_u8_in_set::<1, 2, 4, 8>)?;
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![,]) || lookahead.peek(parse::End) {
            let bit_depth = Pair::new(bit_depth, input.parse()?);
            let sources = Punctuated::parse_terminated(input)?;
            let arguments = Self {
                font,
                weight,
                size,
                hint,
                positions,
                bit_depth,
                sources,
            };

            Ok(arguments)
        } else {
            Err(lookahead.error())
        }
    }
}

fn parse_weight(font: &Font) -> fn(ParseStream) -> syn::Result<u16> {
    use Font::*;

    match font {
        MPLUS1(..) | MPLUS2(..) => |input| {
            let FontWeight::<900>(value) = input.parse()?;

            Ok(value)
        },
        MPLUSCode { .. } => |input| {
            let FontWeight::<700>(value) = input.parse()?;

            Ok(value)
        },
    }
}

fn parse_size(input: ParseStream) -> syn::Result<f32> {
    let FontSize(value) = input.parse()?;

    Ok(value)
}

fn parse_bool(input: ParseStream) -> syn::Result<bool> {
    let syn::LitBool { value, .. } = input.parse()?;

    Ok(value)
}

fn parse_u8_in_range<const MIN: u8, const MAX: u8>(input: ParseStream) -> syn::Result<u8> {
    let lit_int: syn::LitInt = input.parse()?;
    let value = lit_int.base10_parse()?;
    if value < MIN || value > MAX {
        let message = format!("expected number between `{MIN}` and `{MAX}`, found `{value}`");
        return Err(syn::Error::new(lit_int.span(), message));
    }

    Ok(value)
}

fn parse_u8_in_set<const A: u8, const B: u8, const C: u8, const D: u8>(
    input: ParseStream,
) -> syn::Result<u8> {
    let lit_int: syn::LitInt = input.parse()?;
    let value = lit_int.base10_parse()?;
    if ![A, B, C, D].contains(&value) {
        let message = format!("expected one of: `{A}`, `{B}`, `{C}`, `{D}`; found `{value}`");
        return Err(syn::Error::new(lit_int.span(), message));
    }

    Ok(value)
}
