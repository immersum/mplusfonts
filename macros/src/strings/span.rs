use std::ops::Range;
use std::sync::LazyLock;

use proc_macro2::Span;
use regex::Regex;

pub trait SpanExt {
    fn byte_range_shim(&self) -> Range<usize>;
}

impl SpanExt for Span {
    fn byte_range_shim(&self) -> Range<usize> {
        let debug_repr = format!("{self:?}");
        for parse in [parse_proc_macro_server_span, parse_rust_analyzer_span] {
            if let Some(byte_range) = parse(&debug_repr) {
                return byte_range;
            }
        }

        Default::default()
    }
}

fn parse_proc_macro_server_span(debug_repr: &str) -> Option<Range<usize>> {
    const REGEX: &str = r"^#(?-u:\d)+ bytes\(((?-u:\d)+)\.\.((?-u:\d)+)\)";

    static COMPILED_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(REGEX).expect("expected regular expression"));
    let (_, [lo, hi]) = COMPILED_REGEX.captures(debug_repr).map(|c| c.extract())?;
    let range = lo.parse().expect("expected digits")..hi.parse().expect("expected digits");

    Some(range)
}

fn parse_rust_analyzer_span(debug_repr: &str) -> Option<Range<usize>> {
    const REGEX: &str = r"(?-u:\b)range: ((?-u:\d)+)\.\.((?-u:\d)+)";

    static COMPILED_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(REGEX).expect("expected regular expression"));
    let (_, [lo, hi]) = COMPILED_REGEX.captures(debug_repr).map(|c| c.extract())?;
    let range = lo.parse().expect("expected digits")..hi.parse().expect("expected digits");

    Some(range)
}
