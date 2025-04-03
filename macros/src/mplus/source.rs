use std::borrow::Cow;
use std::ops::Bound;

use syn::parse::{Parse, ParseStream};

use super::ExprPathExt;

pub enum CharSource {
    Strings(Vec<String>),
    Range(Bound<char>, Bound<char>),
    Kern(Bound<char>, Bound<char>, Vec<String>),
}

impl CharSource {
    pub fn strings(&self, is_code: bool) -> impl IntoIterator<Item = Cow<[String]>> {
        match *self {
            CharSource::Strings(ref strings) => vec![strings.into()],
            CharSource::Range(start, end) => {
                vec![single_char_strings(start, end).into()]
            }
            CharSource::Kern(start, end, ref strings) if is_code => {
                vec![single_char_strings(start, end).into(), strings.into()]
            }
            CharSource::Kern(Bound::Excluded(start), end, ref strings) if start > '\u{24E}' => {
                vec![
                    single_char_strings(Bound::Excluded(start), end).into(),
                    strings.into(),
                ]
            }
            CharSource::Kern(Bound::Included(start), end, ref strings) if start > '\u{24F}' => {
                vec![
                    single_char_strings(Bound::Included(start), end).into(),
                    strings.into(),
                ]
            }
            CharSource::Kern(start, Bound::Included(end), ref strings) if end < '\u{250}' => {
                vec![
                    single_char_strings(start, Bound::Included(end)).into(),
                    strings_in_cartesian_square(start, Bound::Included(end)).into(),
                    single_char_affixed_strings(start, Bound::Included(end), strings).into(),
                ]
            }
            CharSource::Kern(start, Bound::Excluded(end), ref strings) if end < '\u{251}' => {
                vec![
                    single_char_strings(start, Bound::Excluded(end)).into(),
                    strings_in_cartesian_square(start, Bound::Excluded(end)).into(),
                    single_char_affixed_strings(start, Bound::Excluded(end), strings).into(),
                ]
            }
            CharSource::Kern(start, end, ref strings) => {
                vec![
                    single_char_strings(start, end).into(),
                    strings_in_cartesian_square(start, Bound::Excluded('\u{250}')).into(),
                    single_char_affixed_strings(start, Bound::Excluded('\u{250}'), strings).into(),
                ]
            }
        }
    }
}

impl Parse for CharSource {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let source = match input.parse()? {
            syn::Expr::Array(expr_array) => expr_array.try_into()?,
            syn::Expr::Range(expr_range) => expr_range.try_into()?,
            syn::Expr::Call(expr_call) => expr_call.try_into()?,
            expr => {
                let message = "expected slice literal, range expression, function call expression";
                return Err(syn::Error::new_spanned(expr, message));
            }
        };

        Ok(source)
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

impl TryFrom<syn::ExprCall> for CharSource {
    type Error = syn::Error;

    fn try_from(expr_call: syn::ExprCall) -> Result<Self, Self::Error> {
        let syn::Expr::Path(expr_path) = *expr_call.func else {
            let message = "expected identifier";
            return Err(syn::Error::new_spanned(expr_call.func, message));
        };

        let ident = expr_path.try_into_ident()?;
        let name = ident.to_string();
        if name != "kern" {
            let message = format!("expected identifier `kern`, found `{name}`");
            return Err(syn::Error::new(ident.span(), message));
        }

        let mut exprs = expr_call.args.into_iter();
        let Some(first) = exprs.next() else {
            let message = "expected 2 arguments, found 0";
            return Err(syn::Error::new(expr_call.paren_token.span.join(), message));
        };
        let Some(second) = exprs.next() else {
            let message = "expected 2 arguments, found 1";
            return Err(syn::Error::new(expr_call.paren_token.span.join(), message));
        };
        let syn::Expr::Range(expr_range) = first else {
            let message = "expected range expression";
            return Err(syn::Error::new_spanned(first, message));
        };
        let syn::Expr::Array(expr_array) = second else {
            let message = "expected slice literal expression";
            return Err(syn::Error::new_spanned(second, message));
        };
        let [CharSource::Range(start, end), CharSource::Strings(strings)] =
            [expr_range.try_into()?, expr_array.try_into()?]
        else {
            panic!("expected character range and strings");
        };

        if let Some(third) = exprs.next() {
            let message = "remove the extra argument";
            return Err(syn::Error::new_spanned(third, message));
        }

        Ok(CharSource::Kern(start, end, strings))
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
        $iter.flat_map(|a| $iter.map(move |b| String::from_iter([a, b])))
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

macro_rules! square_strings {
    ($strings:expr) => {
        $strings.into_iter().flat_map(|a| {
            $strings.into_iter().map(move |b| {
                let mut builder = String::with_capacity(a.len() + b.len());
                builder.push_str(a);
                builder.push_str(b);
                builder
            })
        })
    };
}

macro_rules! prefix_strings {
    ($iter:expr, $strings:expr) => {
        $iter.flat_map(|c| {
            $strings.into_iter().map(move |string| {
                let mut builder = String::with_capacity(c.len_utf8() + string.len());
                builder.push(c);
                builder.push_str(string);
                builder
            })
        })
    };
}

macro_rules! suffix_strings {
    ($iter:expr, $strings:expr) => {
        $iter.flat_map(|c| {
            $strings.into_iter().map(move |string| {
                let mut builder = String::with_capacity(c.len_utf8() + string.len());
                builder.push_str(string);
                builder.push(c);
                builder
            })
        })
    };
}

macro_rules! cartesian_products {
    ($iter:expr, $strings:expr) => {
        square_strings!($strings)
            .chain(prefix_strings!($iter, $strings))
            .chain(suffix_strings!($iter, $strings))
    };
}

fn single_char_affixed_strings<'a, T: IntoIterator<Item = &'a String> + Copy>(
    start: Bound<char>,
    end: Bound<char>,
    strings: T,
) -> Vec<String> {
    use Bound::*;

    match (start, end) {
        (Included(start), Included(end)) => cartesian_products!(start..=end, strings).collect(),
        (Included(start), Excluded(end)) => cartesian_products!(start..end, strings).collect(),
        (Included(start), Unbounded) => cartesian_products!(start..=char::MAX, strings).collect(),
        (Unbounded, Included(end)) => cartesian_products!(char::MIN..=end, strings).collect(),
        (Unbounded, Excluded(end)) => cartesian_products!(char::MIN..end, strings).collect(),
        (Unbounded, Unbounded) => cartesian_products!(char::MIN..=char::MAX, strings).collect(),
        (Excluded(_), _) => panic!("expected included or unbounded start index"),
    }
}
