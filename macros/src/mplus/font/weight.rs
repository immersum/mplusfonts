use syn::parse::{Parse, ParseStream};

use super::ExprPathExt;

const CONSTANTS: [(&str, u16); 10] = [
    ("NORMAL", 400),
    ("THIN", 100),
    ("EXTRA_LIGHT", 200),
    ("LIGHT", 300),
    ("REGULAR", 400),
    ("MEDIUM", 500),
    ("SEMI_BOLD", 600),
    ("BOLD", 700),
    ("EXTRA_BOLD", 800),
    ("BLACK", 900),
];

pub struct FontWeight<const MAX: u16>(pub u16);

impl<const MAX: u16> Parse for FontWeight<MAX> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let weight = match input.parse()? {
            syn::Expr::Lit(expr_lit) => expr_lit.try_into()?,
            syn::Expr::Path(expr_path) => expr_path.try_into()?,
            expr => {
                let message = "expected literal or identifier";
                return Err(syn::Error::new_spanned(expr, message));
            }
        };

        Ok(weight)
    }
}

impl<const MAX: u16> TryFrom<syn::ExprLit> for FontWeight<MAX> {
    type Error = syn::Error;

    fn try_from(expr_lit: syn::ExprLit) -> Result<Self, Self::Error> {
        let syn::Lit::Int(lit_int) = expr_lit.lit else {
            let message = "expected integer literal";
            return Err(syn::Error::new_spanned(expr_lit.lit, message));
        };

        let value = lit_int.base10_parse()?;
        if value < 100 || value > MAX {
            let message = format!("expected number between `100` and `{MAX}`, found `{value}`");
            return Err(syn::Error::new(lit_int.span(), message));
        }

        Ok(Self(value))
    }
}

impl<const MAX: u16> TryFrom<syn::ExprPath> for FontWeight<MAX> {
    type Error = syn::Error;

    fn try_from(expr_path: syn::ExprPath) -> Result<Self, Self::Error> {
        let ident = expr_path.try_into_ident()?;
        let name = ident.to_string();
        let mut options = Vec::new();
        for (const_name, value) in CONSTANTS {
            if value < 100 || value > MAX {
                continue;
            }

            if name == const_name {
                return Ok(Self(value));
            }

            options.push(const_name);
        }

        let [first, rest @ ..] = options.as_slice() else {
            panic!("expected `MAX` greater than or equal to `100`, found `{MAX}`");
        };
        let message = if rest.is_empty() {
            format!("expected `{first}`, found `{name}`")
        } else if let [second] = rest {
            format!("expected `{first}` or `{second}`, found `{name}`")
        } else {
            let rest = rest.join("`, `");
            format!("expected `{first}` or one of: `{rest}`; found `{name}`")
        };
        let error = syn::Error::new(ident.span(), message);

        Err(error)
    }
}
