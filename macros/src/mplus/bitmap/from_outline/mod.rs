mod glyph;
mod halfwidth;
mod strings;

use std::collections::BTreeMap;
use std::sync::RwLock;
use std::{iter, thread};

use halfwidth::PixelAlignmentStrategy;
use swash::scale::ScaleContext;
use swash::shape::ShapeContext;

use crate::mplus::Arguments;
use crate::mplus::charmap::CharmapEntry;
use crate::mplus::font::{Font, FontWidth};

use super::CharDictionary;

pub fn render(args: &Arguments, is_fallback: bool) -> BTreeMap<String, CharmapEntry> {
    let entries = BTreeMap::new();
    let font = args.font.value();
    let font_ref = font.as_ref(is_fallback);
    let is_code = matches!(font, Font::MPLUSCode { .. });
    if is_fallback && !is_code {
        return entries;
    }

    let mut coords = Vec::new();
    let units = args.weight.into_value();
    let weight_axis = font_ref
        .variations()
        .find_by_tag(swash::tag_from_str_lossy("wght"))
        .expect("expected font weight axis");

    coords.push(weight_axis.normalize(units.into()));

    let em_per_halfwidth;
    if let Font::MPLUSCode { variable, .. } = font {
        let (.., FontWidth(units)) = *variable;
        if !is_fallback {
            let width_axis = font_ref
                .variations()
                .find_by_tag(swash::tag_from_str_lossy("wdth"))
                .expect("expected font width axis");

            coords.push(width_axis.normalize(units.into()));
        }

        em_per_halfwidth = f32::from(units).mul_add(0.4 / 100.0, 0.1);
    } else {
        em_per_halfwidth = 0.5
    };

    let pixels_per_em = args.size.into_value();
    let glyph_metrics = font_ref.glyph_metrics(&coords).scale(pixels_per_em);

    let pixel_alignment_strategy = PixelAlignmentStrategy::new(pixels_per_em, em_per_halfwidth);

    let mut contexts: Vec<_> = iter::repeat_with(ShapeContext::new)
        .take(thread::available_parallelism().map(Into::into).unwrap_or(1))
        .collect();

    let shapers = contexts.iter_mut().map(|context| {
        context
            .builder(font_ref)
            .normalized_coords(&coords)
            .size(pixels_per_em)
            .features(&[("liga", !is_fallback as u16)])
            .build()
    });

    let positions = args.positions.into_value();
    let bit_depth = args.bit_depth.into_value();

    let mut contexts: Vec<_> = iter::repeat_with(ScaleContext::new)
        .take(shapers.len() * positions as usize)
        .collect();

    let scalers = contexts.iter_mut().map(|context| {
        context
            .builder(font_ref)
            .normalized_coords(&coords)
            .size(pixels_per_em)
            .hint(args.hint.into_value())
            .build()
    });

    let mut scalers: Vec<_> = scalers.collect();
    let scalers = scalers.chunks_mut(positions as usize);
    let renders = scalers.map(|scalers| {
        move |glyph_offsets| {
            glyph::scale(
                scalers,
                is_code,
                pixel_alignment_strategy,
                positions,
                bit_depth,
                &glyph_metrics,
                glyph_offsets,
            )
        }
    });

    let strings: Vec<_> = args
        .sources
        .iter()
        .flat_map(|source| source.strings(is_code))
        .collect();

    let indices = 0..shapers.len();
    let empties = iter::repeat_with(Vec::new).take(shapers.len());
    let strings = strings
        .iter()
        .flat_map(|strings| strings.iter())
        .filter(|string| !string.is_empty())
        .zip(indices.cycle())
        .fold(empties.collect(), |mut strings: Vec<_>, (string, index)| {
            strings
                .get_mut(index)
                .expect("expected index to be less than number of shapers")
                .push(string);

            strings
        });

    let entries = RwLock::new(entries);
    thread::scope(|scope| {
        let entries = CharDictionary::new(&entries);
        shapers
            .into_iter()
            .zip(renders)
            .zip(strings)
            .for_each(|((shaper, render), strings)| {
                scope.spawn(move || {
                    strings::shape_and_render(
                        entries,
                        shaper,
                        render,
                        is_fallback,
                        is_code,
                        pixel_alignment_strategy,
                        strings,
                    )
                });
            });
    });

    entries
        .into_inner()
        .expect("expected no-poison lock on entries")
}
