use std::ops::Div;

use syn::parse::{Parse, ParseStream};

use super::ExprPathExt;

type FontSizeOp = fn(FontSize) -> FontSize;

const FONT_METRICS: [(&str, FontSizeOp); 4] = [
    ("x_height", |px| px / 0.52),
    ("cap_height", |px| px / 0.73),
    ("line_height", |px| px / (1.16 + 0.288)),
    ("code_line_height", |px| px / (1.235 + 0.27)),
];

pub struct FontSize(pub f32);

impl Div<f32> for FontSize {
    type Output = FontSize;

    fn div(mut self, scalar: f32) -> Self::Output {
        self.0 /= scalar;
        self
    }
}

impl Parse for FontSize {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let size = match input.parse()? {
            syn::Expr::Lit(expr_lit) => expr_lit.try_into()?,
            syn::Expr::Call(expr_call) => expr_call.try_into()?,
            expr => {
                let message = "expected literal or function call expression";
                return Err(syn::Error::new_spanned(expr, message));
            }
        };

        Ok(size)
    }
}

impl TryFrom<syn::ExprLit> for FontSize {
    type Error = syn::Error;

    fn try_from(expr_lit: syn::ExprLit) -> Result<Self, Self::Error> {
        let (value, span) = match expr_lit.lit {
            syn::Lit::Int(lit_int) => {
                let value = lit_int.base10_parse()?;
                if value > 0.0 {
                    return Ok(Self(value));
                }

                (value, lit_int.span())
            }
            syn::Lit::Float(lit_float) => {
                let value: f32 = lit_float.base10_parse()?;
                if value > 0.0 {
                    return Ok(Self(value));
                }

                (value, lit_float.span())
            }
            expr_lit => {
                let message = "expected integer or floating point literal";
                return Err(syn::Error::new_spanned(expr_lit, message));
            }
        };
        let message = format!("expected number greater than `0`, found `{value}`");
        let error = syn::Error::new(span, message);

        Err(error)
    }
}

impl TryFrom<syn::ExprCall> for FontSize {
    type Error = syn::Error;

    fn try_from(expr_call: syn::ExprCall) -> Result<Self, Self::Error> {
        let syn::Expr::Path(expr_path) = *expr_call.func else {
            let message = "expected identifier";
            return Err(syn::Error::new_spanned(expr_call.func, message));
        };

        let ident = expr_path.try_into_ident()?;
        let name = ident.to_string();
        let mut options = Vec::new();
        for (fn_name, into_em_size) in FONT_METRICS {
            if name == fn_name {
                let mut exprs = expr_call.args.into_iter();
                let Some(first) = exprs.next() else {
                    let message = "expected 1 argument, found 0";
                    return Err(syn::Error::new(expr_call.paren_token.span.join(), message));
                };
                let syn::Expr::Lit(expr_lit) = first else {
                    let message = "expected literal";
                    return Err(syn::Error::new_spanned(first, message));
                };
                let em_size = expr_lit.try_into().map(into_em_size)?;

                if let Some(second) = exprs.next() {
                    let message = "remove the extra argument";
                    return Err(syn::Error::new_spanned(second, message));
                }

                return Ok(em_size);
            }

            options.push(fn_name);
        }

        let options = options.join(", ");
        let message = format!("expected one of: {options}");
        let error = syn::Error::new(ident.span(), message);

        Err(error)
    }
}
