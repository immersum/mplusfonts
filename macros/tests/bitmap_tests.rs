#![cfg(not(feature = "bless-tests"))]

use embedded_graphics::prelude::*;
use mplusfonts::CharmapEntry;
use mplusfonts::glyph::Glyph;
use mplusfonts::image::Colors;
use mplusfonts_macros::mplus;
use seq_macro::seq;

macro_rules! test_render_glyphs {
    (
        $(
            $fn_ident:ident, $bitmap_font:expr, $text_fragments:expr, $expected_data:expr,
        )*
    ) => {
        $(
            #[test]
            fn $fn_ident() {
                let bitmap_font = $bitmap_font;
                let text_fragments = $text_fragments;
                let (advance_widths, gray_values) = $expected_data;
                let expected_data = advance_widths.into_iter().zip(gray_values);

                for (slice, tuple) in text_fragments.into_iter().zip(expected_data) {
                    let entry = bitmap_font.charmap.get(slice);
                    let CharmapEntry {
                        key,
                        advance_chars,
                        advance_width_to,
                        glyph:
                            Glyph {
                                id,
                                ref images,
                                next,
                            },
                    } = *entry;

                    if id == 0 {
                        continue;
                    }

                    assert_eq!(slice, key, "\n slice: {slice:?}");
                    assert_eq!(slice.chars().count(), advance_chars, "\n slice: {slice:?}");

                    let (advance_widths, gray_values) = tuple;
                    for (slice, expected) in text_fragments.into_iter().zip(advance_widths) {
                        let result = advance_width_to(slice);
                        assert_eq!(
                            result,
                            *expected,
                            "\n slice: {slice:?}",
                            slice = key.to_owned() + slice
                        );
                    }

                    let image = images.get(0);
                    let width = image.bounding_box().size.width;
                    let mut colors = image.colors().into_iter();
                    let mut index = 0;
                    for expected in gray_values {
                        let Some(result) = colors.next().map(IntoStorage::into_storage) else {
                            panic!("expected a gray value\n slice: {slice:?}\n index: {index:?}");
                        };
                        assert_eq!(
                            result,
                            *expected,
                            "\n slice: {slice:?}\n  x, y: {x:?}, {y:?}",
                            x = index % width,
                            y = index / width
                        );

                        index += 1;
                    }

                    assert!(
                        colors.next().is_none(),
                        "unexpected gray value\n slice: {slice:?}\n index: {index:?}"
                    );

                    assert!(next.is_none(), "unexpected next glyph\n slice: {slice:?}");
                }
            }
        )*
    }
}

test_render_glyphs! {
    render_glyphs_1_500_25_false_1_4_kern_space_tilde_liga,
        mplus!(1, 500, 25, false, 1, 4, kern(' '..='~', ["ffi", "ffl"])),
        seq!(C in ' '..='~' { [#(concat!(C),)* "ff", "fi", "ffi", "fl", "ffl"] }),
        include!("bitmap/render_glyphs_1_500_25_false_1_4_kern_space_tilde_liga.in"),

    render_glyphs_1_500_25_false_1_4_kern_latin_1_sup_liga,
        mplus!(1, 500, 25, false, 1, 4, kern(' '..='ÿ', ["ffi", "ffl"])),
        seq!(C in ' '..='ÿ' { [#(concat!(C),)* "ff", "fi", "ffi", "fl", "ffl"] }),
        include!("bitmap/render_glyphs_1_500_25_false_1_4_kern_latin_1_sup_liga.in"),

    render_glyphs_2_500_25_false_1_4_kern_latin_ext_a_liga,
        mplus!(2, 500, 25, false, 1, 4, kern('Ā'..'ƀ', ["ffi", "ffl"])),
        seq!(C in 'Ā'..'ƀ' { [#(concat!(C),)* "ff", "fi", "ffi", "fl", "ffl"] }),
        include!("bitmap/render_glyphs_2_500_25_false_1_4_kern_latin_ext_a_liga.in"),

    render_glyphs_2_500_25_false_1_4_kern_latin_ext_b_liga,
        mplus!(2, 500, 25, false, 1, 4, kern('ƀ'..'ɐ', ["ffi", "ffl"])),
        seq!(C in 'ƀ'..'ɐ' { [#(concat!(C),)* "ff", "fi", "ffi", "fl", "ffl"] }),
        include!("bitmap/render_glyphs_2_500_25_false_1_4_kern_latin_ext_b_liga.in"),

    render_glyphs_code_100_500_25_false_1_4_hiragana_kanji,
        mplus!(code(100), 500, 25, false, 1, 4, 'ぁ'..='ゖ', 'ゝ'..='ゟ', ["東京", "京都"]),
        seq!(C in 'ぁ'..='ゟ' { [#(concat!(C),)* "東", "京", "都"] }),
        include!("bitmap/render_glyphs_code_100_500_25_false_1_4_hiragana_kanji.in"),

    render_glyphs_code_125_500_25_false_1_4_katakana_kanji,
        mplus!(code(125), 500, 25, false, 1, 4, 'ァ'..='ヺ', 'ヽ'..='ヿ', ["東京", "京都"]),
        seq!(C in 'ァ'..='ヿ' { [#(concat!(C),)* "東", "京", "都"] }),
        include!("bitmap/render_glyphs_code_125_500_25_false_1_4_katakana_kanji.in"),

    render_glyphs_1_500_25_false_1_4_kern_dotless_i_j_liga,
        mplus!(1, 500, 25, false, 1, 4, kern('i'..='j', ["ı", "ȷ", "f", "ff", "fi", "ffi"])),
        seq!(C in 'i'..='j' { [#(concat!(C),)* "ı", "ȷ", "f", "ff", "fi", "ffi"] }),
        include!("bitmap/render_glyphs_1_500_25_false_1_4_kern_dotless_i_j_liga.in"),
}
