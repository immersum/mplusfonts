use syn::punctuated::Pair;
use syn::spanned::Spanned;

pub trait ExprPathExt {
    fn try_into_ident(self) -> syn::Result<syn::Ident>;
}

impl ExprPathExt for syn::ExprPath {
    fn try_into_ident(mut self) -> syn::Result<syn::Ident> {
        if let Some(ref qself) = self.qself {
            let message = "unexpected path qualifier";
            return Err(syn::Error::new(qself.span(), message));
        }

        if let Some(colon) = self.path.leading_colon {
            let message = "remove the leading colon";
            return Err(syn::Error::new_spanned(colon, message));
        }

        let segment = match self.path.segments.pop().expect("expected path segment") {
            Pair::Punctuated(_, colon) => {
                let message = "remove the trailing colon";
                return Err(syn::Error::new_spanned(colon, message));
            }
            Pair::End(segment) => segment,
        };

        if !self.path.segments.is_empty() {
            let message = "unexpected path segments";
            return Err(syn::Error::new_spanned(self.path.segments, message));
        }

        if !segment.arguments.is_empty() {
            let message = "unexpected path arguments";
            return Err(syn::Error::new_spanned(segment.arguments, message));
        }

        Ok(segment.ident)
    }
}
