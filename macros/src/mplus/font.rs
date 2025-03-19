mod size;
mod weight;
mod width;

use lazy_static_include::lazy_static_include_bytes;
use swash::{CacheKey, FontRef};
use syn::parse::{Parse, ParseStream};

use super::ExprPathExt;

pub use size::FontSize;
pub use weight::FontWeight;
pub use width::FontWidth;

lazy_static_include_bytes! {
    MPLUS1 => "fonts/MPLUS1[wght].ttf",
    MPLUS2 => "fonts/MPLUS2[wght].ttf",
    MPLUS1Code => "fonts/MPLUS1Code[wght].ttf",
    MPLUSCodeLatin => "fonts/MPLUSCodeLatin[wdth,wght].ttf",
}

pub enum Font {
    MPLUS1(u32, CacheKey),
    MPLUS2(u32, CacheKey),
    MPLUSCode {
        variable: (u32, CacheKey, FontWidth<125>),
        fallback: (u32, CacheKey),
    },
}

impl Font {
    pub fn as_ref(&self, is_fallback: bool) -> FontRef {
        let (data, offset, key) = match *self {
            Self::MPLUS1(offset, key) => (MPLUS1.as_ref(), offset, key),
            Self::MPLUS2(offset, key) => (MPLUS2.as_ref(), offset, key),
            Self::MPLUSCode {
                fallback: (offset, key),
                ..
            } if is_fallback => (MPLUS1Code.as_ref(), offset, key),
            Self::MPLUSCode {
                variable: (offset, key, _),
                ..
            } => (MPLUSCodeLatin.as_ref(), offset, key),
        };

        FontRef { data, offset, key }
    }
}

impl Parse for Font {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let font = match input.parse()? {
            syn::Expr::Lit(expr_lit) => expr_lit.try_into()?,
            syn::Expr::Call(expr_call) => expr_call.try_into()?,
            expr => {
                let message = "expected literal or function call expression";
                return Err(syn::Error::new_spanned(expr, message));
            }
        };

        Ok(font)
    }
}

impl TryFrom<syn::ExprLit> for Font {
    type Error = syn::Error;

    fn try_from(expr_lit: syn::ExprLit) -> Result<Self, Self::Error> {
        let syn::Lit::Int(lit_int) = expr_lit.lit else {
            let message = "expected integer literal";
            return Err(syn::Error::new_spanned(expr_lit.lit, message));
        };

        let font = match lit_int.base10_digits() {
            "1" => {
                let font_ref = FontRef::from_index(&MPLUS1, 0).expect("expected font");
                Self::MPLUS1(font_ref.offset, font_ref.key)
            }
            "2" => {
                let font_ref = FontRef::from_index(&MPLUS2, 0).expect("expected font");
                Self::MPLUS2(font_ref.offset, font_ref.key)
            }
            digits => {
                let message = format!("expected number `1` or `2`, found `{digits}`");
                return Err(syn::Error::new(lit_int.span(), message));
            }
        };

        Ok(font)
    }
}

impl TryFrom<syn::ExprCall> for Font {
    type Error = syn::Error;

    fn try_from(expr_call: syn::ExprCall) -> Result<Self, Self::Error> {
        let syn::Expr::Path(expr_path) = *expr_call.func else {
            let message = "expected identifier";
            return Err(syn::Error::new_spanned(expr_call.func, message));
        };

        let ident = expr_path.try_into_ident()?;
        let name = ident.to_string();
        if name != "code" {
            let message = format!("expected identifier `code`, found `{name}`");
            return Err(syn::Error::new(ident.span(), message));
        }

        let mut exprs = expr_call.args.into_iter();
        let Some(first) = exprs.next() else {
            let message = "expected 1 argument, found 0";
            return Err(syn::Error::new(expr_call.paren_token.span.join(), message));
        };
        let width = first.try_into()?;

        if let Some(second) = exprs.next() {
            let message = "remove the extra argument";
            return Err(syn::Error::new_spanned(second, message));
        }

        let variable_font_ref = FontRef::from_index(&MPLUSCodeLatin, 0).expect("expected font");
        let fallback_font_ref = FontRef::from_index(&MPLUS1Code, 0).expect("expected font");
        let font = Self::MPLUSCode {
            variable: (variable_font_ref.offset, variable_font_ref.key, width),
            fallback: (fallback_font_ref.offset, fallback_font_ref.key),
        };

        Ok(font)
    }
}
