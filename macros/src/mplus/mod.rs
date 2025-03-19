mod args;
mod bitmap;
mod charmap;
mod expr;
mod font;
mod source;

use std::collections::BTreeMap;

use bitmap::{BitmapFont, render_glyphs};
use charmap::Charmap;
use expr::ExprPathExt;
use font::Font;
use proc_macro2::TokenStream;
use quote::ToTokens;
use source::CharSource;

pub use args::Arguments;

pub fn mplus_impl(mut args: Arguments) -> TokenStream {
    let notdef = CharSource::Strings(vec![String::from("\u{FFFD}")]);
    args.sources.push(notdef);

    let mut entries = BTreeMap::new();
    entries.extend(render_glyphs(&args, true));
    entries.extend(render_glyphs(&args, false));

    let notdef = entries.remove("\u{FFFD}").expect("expected `\u{FFFD}`");
    let charmap = Charmap::from_iter(entries);
    let positions = args.positions.into_value();
    let bit_depth = args.bit_depth.into_value();
    let size = args.size.into_value();
    let is_code = matches!(args.font.into_value(), Font::MPLUSCode { .. });
    let font = BitmapFont {
        charmap,
        notdef,
        positions,
        bit_depth,
        size,
        is_code,
    };

    font.into_token_stream()
}
