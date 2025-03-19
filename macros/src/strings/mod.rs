mod attr;
mod item;
mod span;
mod visitor;

use std::ops::Range;

use attr::AttrExt;
use item::ItemExt;
use proc_macro2::{Literal, TokenStream};
use quote::{ToTokens, quote};
use span::SpanExt;
use syn::spanned::Spanned;
use syn::visit_mut::VisitMut;
use visitor::lit::LitStrVisitor;
use visitor::mac::MacroVisitor;

pub fn strings_impl(mut item: syn::Item) -> TokenStream {
    let mut tokens = TokenStream::new();
    let mut lit_strings = Vec::new();
    let mut visitor = LitStrVisitor::new();
    visitor.visit_item_mut(&mut item);
    match visitor.into_result() {
        Ok(strings) => {
            if cfg!(feature = "doc-comment") {
                for string in strings.iter() {
                    lit_strings.push(Literal::string(string));
                }
            }

            let mut visitor = MacroVisitor::new(strings);
            visitor.visit_item_mut(&mut item);
            if let Err(e) = visitor.into_result() {
                tokens.extend(e.to_compile_error());
            }
        }
        Err(e) => {
            tokens.extend(e.to_compile_error());
        }
    }

    if cfg!(feature = "doc-comment") {
        let variant_ident = item.variant_ident();
        let Range { start, .. } = item.span().byte_range_shim();
        let strings = quote!([#(#lit_strings),*]);
        let comment = format!("``strings_from_{variant_ident}@byte({start}) = {strings}``");
        tokens.extend(quote!(#[doc = #comment]));
    }

    tokens.extend(item.to_token_stream());
    tokens
}
