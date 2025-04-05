#![cfg(feature = "bless-tests")]

use std::error::Error;
use std::fs::File;
use std::io::Write;

use embedded_graphics::prelude::*;
use mplusfonts::CharmapEntry;
use mplusfonts::image::Colors;
use mplusfonts_macros::mplus;
use seq_macro::seq;

macro_rules! bless_render_glyphs {
    (
        $(
            $fn_ident:ident, $bitmap_font:expr, $text_fragments:expr, $path:expr,
        )*
    ) => {
        $(
            #[test]
            fn $fn_ident() -> Result<(), Box<dyn Error>> {
                let bitmap_font = $bitmap_font;
                let text_fragments = $text_fragments;
                let mut expected_data = File::create($path)?;
                write!(expected_data, "(")?;
                write!(expected_data, "\n    [")?;

                let mut slice_index = 0;
                for slice in text_fragments {
                    let entry = bitmap_font.charmap.get(slice);
                    let CharmapEntry {
                        advance_width_to, ..
                    } = *entry;
                    let advance_widths = text_fragments.map(advance_width_to);
                    write!(expected_data, "\n        // {slice:?}")?;
                    write!(expected_data, "\n        &[")?;

                    let mut value_index = 0;
                    for value in advance_widths {
                        if value_index % 10 == 0 {
                            write!(expected_data, "\n            ")?;
                        } else {
                            write!(expected_data, " ")?;
                        }

                        if value_index % 10 > 0 {
                            write!(expected_data, "{value:?},")?;
                        } else {
                            write!(expected_data, "{value:?}f32,")?;
                        }

                        value_index += 1;
                    }

                    if slice_index > 0 {
                        write!(expected_data, "\n        ],")?;
                    } else {
                        write!(expected_data, "\n        ][..],")?;
                    }

                    slice_index += 1;
                }

                write!(expected_data, "\n    ],")?;
                write!(expected_data, "\n    [")?;

                let mut slice_index = 0;
                for slice in text_fragments {
                    let entry = bitmap_font.charmap.get(slice);
                    let image = entry.glyph.images.get(0);
                    let width = image.bounding_box().size.width;
                    let colors = image.colors().into_iter();
                    let gray_values = colors.map(IntoStorage::into_storage);
                    write!(expected_data, "\n        // {slice:?}")?;
                    write!(expected_data, "\n        &[")?;

                    let mut value_index = 0;
                    for value in gray_values {
                        if value_index % width == 0 {
                            write!(expected_data, "\n            ")?;
                        } else {
                            write!(expected_data, " ")?;
                        }

                        if value_index % width > 0 {
                            write!(expected_data, "{value:?},")?;
                        } else {
                            write!(expected_data, "{value:?}u8,")?;
                        }

                        value_index += 1;
                    }

                    if slice_index > 0 {
                        write!(expected_data, "\n        ],")?;
                    } else {
                        write!(expected_data, "\n        ][..],")?;
                    }

                    slice_index += 1;
                }

                write!(expected_data, "\n    ]")?;
                write!(expected_data, "\n)")?;

                Ok(())
            }
        )*
    }
}

bless_render_glyphs! {
    render_glyphs_1_500_25_false_1_4_kern_space_tilde_liga,
        mplus!(1, 500, 25, false, 1, 4, kern(' '..='~', ["ffi", "ffl"])),
        seq!(C in ' '..='~' { [#(concat!(C),)* "ff", "fi", "ffi", "fl", "ffl"] }),
        "tests/bitmap/render_glyphs_1_500_25_false_1_4_kern_space_tilde_liga.in",

    render_glyphs_1_500_25_false_1_4_kern_latin_1_sup_liga,
        mplus!(1, 500, 25, false, 1, 4, kern(' '..='ÿ', ["ffi", "ffl"])),
        seq!(C in ' '..='ÿ' { [#(concat!(C),)* "ff", "fi", "ffi", "fl", "ffl"] }),
        "tests/bitmap/render_glyphs_1_500_25_false_1_4_kern_latin_1_sup_liga.in",

    render_glyphs_2_500_25_false_1_4_kern_latin_ext_a_liga,
        mplus!(2, 500, 25, false, 1, 4, kern('Ā'..'ƀ', ["ffi", "ffl"])),
        seq!(C in 'Ā'..'ƀ' { [#(concat!(C),)* "ff", "fi", "ffi", "fl", "ffl"] }),
        "tests/bitmap/render_glyphs_2_500_25_false_1_4_kern_latin_ext_a_liga.in",

    render_glyphs_2_500_25_false_1_4_kern_latin_ext_b_liga,
        mplus!(2, 500, 25, false, 1, 4, kern('ƀ'..'ɐ', ["ffi", "ffl"])),
        seq!(C in 'ƀ'..'ɐ' { [#(concat!(C),)* "ff", "fi", "ffi", "fl", "ffl"] }),
        "tests/bitmap/render_glyphs_2_500_25_false_1_4_kern_latin_ext_b_liga.in",

    render_glyphs_code_100_500_25_false_1_4_hiragana_kanji,
        mplus!(code(100), 500, 25, false, 1, 4, 'ぁ'..='ゖ', 'ゝ'..='ゟ', ["東京", "京都"]),
        seq!(C in 'ぁ'..='ゟ' { [#(concat!(C),)* "東", "京", "都"] }),
        "tests/bitmap/render_glyphs_code_100_500_25_false_1_4_hiragana_kanji.in",

    render_glyphs_code_125_500_25_false_1_4_katakana_kanji,
        mplus!(code(125), 500, 25, false, 1, 4, 'ァ'..='ヺ', 'ヽ'..='ヿ', ["東京", "京都"]),
        seq!(C in 'ァ'..='ヿ' { [#(concat!(C),)* "東", "京", "都"] }),
        "tests/bitmap/render_glyphs_code_125_500_25_false_1_4_katakana_kanji.in",

    render_glyphs_1_500_25_false_1_4_kern_dotless_i_j_liga,
        mplus!(1, 500, 25, false, 1, 4, kern('i'..='j', ["ı", "ȷ", "f", "ff", "fi", "ffi"])),
        seq!(C in 'i'..='j' { [#(concat!(C),)* "ı", "ȷ", "f", "ff", "fi", "ffi"] }),
        "tests/bitmap/render_glyphs_1_500_25_false_1_4_kern_dotless_i_j_liga.in",
}
