use std::borrow::Cow;
use std::ops::Bound;

use syn::parse::{Parse, ParseStream};

pub enum CharSource {
    Strings(Vec<String>),
    Range(Bound<char>, Bound<char>),
}

impl CharSource {
    pub fn strings(&self, is_code: bool) -> Cow<[String]> {
        match *self {
            CharSource::Strings(ref strings) => Cow::from(strings),
            CharSource::Range(start, end) if is_code => Cow::from(single_char_strings(start, end)),
            CharSource::Range(Bound::Excluded(start), end) if start > '\u{24E}' => {
                Cow::from(single_char_strings(Bound::Excluded(start), end))
            }
            CharSource::Range(Bound::Included(start), end) if start > '\u{24F}' => {
                Cow::from(single_char_strings(Bound::Included(start), end))
            }
            CharSource::Range(start, Bound::Included(end)) if end < '\u{250}' => {
                Cow::from(strings_in_cartesian_square(start, Bound::Included(end)))
            }
            CharSource::Range(start, Bound::Excluded(end)) if end < '\u{251}' => {
                Cow::from(strings_in_cartesian_square(start, Bound::Excluded(end)))
            }
            CharSource::Range(start, end) => {
                let mut strings = strings_in_cartesian_square(start, Bound::Excluded('\u{250}'));
                strings.append(single_char_strings(Bound::Included('\u{250}'), end).as_mut());

                Cow::from(strings)
            }
        }
    }
}

impl Parse for CharSource {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let strings = match input.parse()? {
            syn::Expr::Array(expr_array) => expr_array.try_into()?,
            syn::Expr::Range(expr_range) => expr_range.try_into()?,
            expr => {
                let message = "expected slice literal or range expression";
                return Err(syn::Error::new_spanned(expr, message));
            }
        };

        Ok(strings)
    }
}

impl TryFrom<syn::ExprArray> for CharSource {
    type Error = syn::Error;

    fn try_from(expr_array: syn::ExprArray) -> Result<Self, Self::Error> {
        let exprs = expr_array.elems.into_iter();
        let strings = exprs.map(|expr| {
            let syn::Expr::Lit(expr_lit) = expr else {
                let message = "expected literal";
                return Err(syn::Error::new_spanned(expr, message));
            };

            let syn::Lit::Str(lit_str) = expr_lit.lit else {
                let message = "expected string literal";
                return Err(syn::Error::new_spanned(expr_lit.lit, message));
            };

            let value = lit_str.value();

            Ok(value)
        });
        let strings: Result<Vec<_>, _> = strings.collect();

        Ok(Self::Strings(strings?))
    }
}

impl TryFrom<syn::ExprRange> for CharSource {
    type Error = syn::Error;

    fn try_from(expr_range: syn::ExprRange) -> Result<Self, Self::Error> {
        use syn::RangeLimits::*;

        let exprs = [expr_range.start, expr_range.end];
        let [start, end] = exprs.map(|expr| {
            expr.map(|expr| {
                let syn::Expr::Lit(expr_lit) = *expr else {
                    let message = "expected literal";
                    return Err(syn::Error::new_spanned(expr, message));
                };

                let syn::Lit::Char(lit_char) = expr_lit.lit else {
                    let message = "expected character literal";
                    return Err(syn::Error::new_spanned(expr_lit.lit, message));
                };

                let value = lit_char.value();

                Ok(value)
            })
        });
        let start = start
            .transpose()
            .map(|c| c.map(Bound::Included).unwrap_or(Bound::Unbounded));

        let into_bound = match expr_range.limits {
            HalfOpen(_) => Bound::Excluded,
            Closed(_) => Bound::Included,
        };
        let end = end
            .transpose()
            .map(|c| c.map(into_bound).unwrap_or(Bound::Unbounded));

        Ok(Self::Range(start?, end?))
    }
}

fn single_char_strings(start: Bound<char>, end: Bound<char>) -> Vec<String> {
    use Bound::*;

    match (start, end) {
        (Included(start), Included(end)) => (start..=end).map(|c| c.to_string()).collect(),
        (Included(start), Excluded(end)) => (start..end).map(|c| c.to_string()).collect(),
        (Included(start), Unbounded) => (start..=char::MAX).map(|c| c.to_string()).collect(),
        (Unbounded, Included(end)) => (char::MIN..=end).map(|c| c.to_string()).collect(),
        (Unbounded, Excluded(end)) => (char::MIN..end).map(|c| c.to_string()).collect(),
        (Unbounded, Unbounded) => (char::MIN..=char::MAX).map(|c| c.to_string()).collect(),
        (Excluded(_), _) => panic!("expected included or unbounded start index"),
    }
}

macro_rules! cartesian_square {
    ($iter:expr) => {
        $iter.flat_map(|a| $iter.map(move |b| String::from_iter([a, '\u{200C}', b])))
    };
}

fn strings_in_cartesian_square(start: Bound<char>, end: Bound<char>) -> Vec<String> {
    use Bound::*;

    match (start, end) {
        (Included(start), Included(end)) => cartesian_square!(start..=end).collect(),
        (Included(start), Excluded(end)) => cartesian_square!(start..end).collect(),
        (Included(start), Unbounded) => cartesian_square!(start..=char::MAX).collect(),
        (Unbounded, Included(end)) => cartesian_square!(char::MIN..=end).collect(),
        (Unbounded, Excluded(end)) => cartesian_square!(char::MIN..end).collect(),
        (Unbounded, Unbounded) => cartesian_square!(char::MIN..=char::MAX).collect(),
        (Excluded(_), _) => panic!("expected included or unbounded start index"),
    }
}
