mod color;
mod font;
mod glyph;
mod image;

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::sync::{Mutex, RwLock};
use std::{iter, thread};

use swash::GlyphMetrics;
use swash::scale::{Render, ScaleContext, Scaler, Source};
use swash::shape::{ShapeContext, Shaper};
use swash::text::cluster::SourceRange;
use swash::zeno::Vector;

use super::Arguments;
use super::charmap::CharmapEntry;
use super::font::{Font, FontWidth};

pub use font::BitmapFont;
pub use glyph::{Glyph, GlyphList};
pub use image::{Image, ImageList};

#[derive(Clone, Copy)]
struct Entries<'a>(&'a RwLock<BTreeMap<String, CharmapEntry>>);

#[derive(Clone, Copy)]
struct Halfwidth(f32);

#[derive(Clone, Copy)]
enum PixelAlignmentStrategy {
    Floor(Halfwidth),
    Ceil,
    Zero,
}

pub fn render_glyphs(args: &Arguments, is_fallback: bool) -> BTreeMap<String, CharmapEntry> {
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

    let pixel_alignment_strategy = match pixels_per_em {
        ..1.25 => PixelAlignmentStrategy::Zero,
        ..2.0 => PixelAlignmentStrategy::Ceil,
        _ => PixelAlignmentStrategy::Floor(Halfwidth(pixels_per_em * em_per_halfwidth)),
    };

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
            scale_glyph(
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
        let entries = Entries(&entries);
        shapers
            .into_iter()
            .zip(renders)
            .zip(strings)
            .for_each(|((shaper, render), strings)| {
                scope.spawn(move || {
                    shape_and_render_strings(
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

fn try_to_entry_key(string: &str, start: usize, end: usize) -> Result<String, ()> {
    let bytes: Vec<_> = string.bytes().skip(start).take(end - start).collect();

    debug_assert!(
        !bytes.is_empty(),
        "indexing into `{string:?}`, out of bounds at `{end:?}`"
    );
    let entry_key = match String::from_utf8(bytes) {
        Ok(substring) if substring.is_empty() => return Err(()),
        Ok(substring) => substring,
        Err(e) => {
            let message = format!("expected character boundary at bytes `{start:?}` and `{end:?}`");
            debug_assert_eq!(None, Some(e), "indexing into `{string:?}`, {message}");
            return Err(());
        }
    };

    Ok(entry_key)
}

fn shape_and_render_strings(
    entries: Entries,
    mut shaper: Shaper,
    mut render: impl FnMut((u16, f32, f32)) -> Glyph,
    is_fallback: bool,
    is_code: bool,
    pixel_alignment_strategy: PixelAlignmentStrategy,
    strings: Vec<&String>,
) {
    for string in strings.iter() {
        shaper.add_str(string);
        shaper.add_str("\n");
    }

    let mut strings = strings.into_iter();
    let mut string = "";
    let mut newline = false;
    let mut previous = None;
    shaper.shape_with(|glyph_cluster| {
        let SourceRange { start, end } = glyph_cluster.source;
        if start == 0 {
            newline = !newline;
            if newline {
                string = strings.next().expect("expected string iterator to yield");
            } else {
                return;
            }
        }

        let Ok(entry_key) = try_to_entry_key(string, start as usize, end as usize) else {
            return;
        };
        let (glyphs, mut advance_width) = glyph_cluster
            .glyphs
            .iter()
            .filter(|glyph| glyph.id > 0 || entry_key == "\u{FFFD}")
            .fold((Vec::new(), 0.0), |(mut glyphs, advance_width), glyph| {
                let glyph_offsets = (glyph.id, glyph.x, glyph.y);
                let advance_width = advance_width + glyph.advance;
                glyphs.push(glyph_offsets);

                (glyphs, advance_width)
            });

        let advance_width_adjustment;
        if is_code {
            match pixel_alignment_strategy {
                PixelAlignmentStrategy::Floor(halfwidth) => {
                    advance_width = advance_width.floor();
                    advance_width_adjustment = halfwidth.adjustment(advance_width);
                }
                PixelAlignmentStrategy::Ceil => {
                    advance_width = (advance_width / 1.2).ceil();
                    advance_width_adjustment = 0.0;
                }
                PixelAlignmentStrategy::Zero => {
                    advance_width = 0.0;
                    advance_width_adjustment = 0.0;
                }
            }
        } else {
            advance_width_adjustment = 0.0;
        }

        advance_width += advance_width_adjustment;

        if !glyphs.is_empty() || glyph_cluster.is_empty() {
            if !entries.contains_key(&entry_key) {
                let glyphs = glyphs.into_iter().map(&mut render);
                entries.insert_glyphs(entry_key.clone(), glyphs.collect());
            }

            if is_fallback {
                return;
            }

            let to_entry_key = entry_key.clone();
            if let Some((entry_key, advance_width)) = previous.replace((entry_key, advance_width)) {
                entries.insert_advance_width(entry_key, to_entry_key, advance_width);
            }
        } else {
            previous = None;
        }
    });
}

fn scale_glyph(
    scalers: &mut [Scaler],
    is_code: bool,
    pixel_alignment_strategy: PixelAlignmentStrategy,
    positions: u8,
    bit_depth: u8,
    glyph_metrics: &GlyphMetrics,
    glyph_offsets: (u16, f32, f32),
) -> Glyph {
    debug_assert_eq!(scalers.len(), positions as usize);
    let (id, mut x_offset, mut y_offset) = glyph_offsets;
    let mut advance_width = glyph_metrics.advance_width(id);
    let mut advance_height = glyph_metrics.advance_height(id);
    let advance_width_adjustment;
    if is_code {
        match pixel_alignment_strategy {
            PixelAlignmentStrategy::Floor(halfwidth) => {
                advance_width = advance_width.floor();
                advance_width_adjustment = halfwidth.adjustment(advance_width);
                advance_height = advance_height.floor();
                x_offset = x_offset.ceil();
            }
            PixelAlignmentStrategy::Ceil => {
                advance_width = (advance_width / 1.2).ceil();
                advance_width_adjustment = 0.0;
                advance_height = advance_height.ceil();
                x_offset = x_offset.ceil();
            }
            PixelAlignmentStrategy::Zero => {
                advance_width = 0.0;
                advance_width_adjustment = 0.0;
                advance_height = 0.0;
                x_offset = 0.0;
            }
        }
    } else {
        advance_width_adjustment = 0.0;
    }

    if [812, 813, 814, 815, 817, 818, 819, 820, 821, 823, 825].contains(&id) {
        x_offset = 0.0;
        y_offset = 0.0;
    }

    let is_square = advance_width == advance_height || id == 0;
    let positions = if is_code || is_square { 1 } else { positions };
    let x_padding = (advance_width_adjustment / 2.0) as u32;
    advance_width += advance_width_adjustment;

    let images = Mutex::new(BTreeMap::new());
    if advance_width > 0.0 {
        thread::scope(|scope| {
            let images = &images;
            (0..positions).zip(scalers).for_each(|(index, scaler)| {
                scope.spawn(move || {
                    let x_offset = f32::from(index) / f32::from(positions);
                    let image = Render::new(&[Source::Outline])
                        .offset(Vector::new(x_offset, 0.0))
                        .render(scaler, id)
                        .expect("expected glyph outline");

                    if image.data.is_empty() {
                        return;
                    }

                    let image = Image {
                        left: image.placement.left.saturating_add_unsigned(x_padding),
                        top: image.placement.top,
                        width: image.placement.width,
                        data: color::quantize(&image.data, image.placement.width, bit_depth),
                    };

                    let mut images = images.lock().expect("expected no-poison lock on images");
                    let image = images.insert(index, image);
                    debug_assert!(image.is_none(), "expected not to remove an existing image");
                });
            });
        });
    }

    let images = images
        .into_inner()
        .expect("expected no-poison lock on images");

    let images: Vec<_> = images.into_values().collect();
    if !images.is_empty() {
        debug_assert_eq!(images.len(), positions as usize);
    }

    Glyph {
        x_offset,
        y_offset,
        positions,
        bit_depth,
        id,
        advance_width,
        images: ImageList(images),
    }
}

impl Entries<'_> {
    pub fn contains_key(self, entry_key: &String) -> bool {
        let Entries(entries) = self;
        let entries = entries.read().expect("expected no-poison lock on entries");

        entries.contains_key(entry_key)
    }

    pub fn insert_glyphs(self, entry_key: String, glyphs: Vec<Glyph>) {
        let Entries(entries) = self;
        let mut entries = entries.write().expect("expected no-poison lock on entries");
        let entry = entries.entry(entry_key);
        entry.or_insert_with_key(|key| CharmapEntry {
            key: key.clone(),
            advance_chars: key.chars().count(),
            advance_width_to: BTreeMap::new(),
            advance_width: glyphs.iter().map(|glyph| glyph.advance_width).sum(),
            glyphs: GlyphList(glyphs),
        });
    }

    pub fn insert_advance_width(self, entry_key: String, to_entry_key: String, advance_width: f32) {
        let Entries(entries) = self;
        let mut entries = entries.write().expect("expected no-poison lock on entries");
        let entry = entries.entry(entry_key);
        let entry = entry.and_modify(|entry| {
            if entry.advance_width != advance_width {
                if let Some(previous) = entry.advance_width_to.insert(to_entry_key, advance_width) {
                    debug_assert_eq!(
                        previous,
                        advance_width,
                        "expected no change in value in case of update to entry with key `{key:?}`",
                        key = entry.key,
                    );
                }
            }
        });
        debug_assert!(
            matches!(entry, Entry::Occupied(_)),
            "expected to modify an existing entry"
        );
    }
}

impl Halfwidth {
    fn adjustment(&self, advance_width: f32) -> f32 {
        let Self(halfwidth) = *self;
        debug_assert_ne!(0.0, halfwidth);
        debug_assert_eq!(advance_width.signum(), halfwidth.signum());

        let mut new_advance_width = halfwidth.floor();
        while advance_width > new_advance_width + halfwidth * 0.4 {
            new_advance_width += halfwidth;
        }

        new_advance_width = new_advance_width.floor();
        new_advance_width - advance_width
    }
}
