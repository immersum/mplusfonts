use swash::shape::cluster::Glyph;

pub enum GlyphOffsets {
    Initial(u16, f32, f32),
    Overlay(u16, f32, f32),
}

impl GlyphOffsets {
    pub fn new(is_overlay: bool, id: u16, x_offset: f32, y_offset: f32) -> Self {
        if is_overlay {
            Self::Overlay(id, x_offset, y_offset)
        } else {
            Self::Initial(id, x_offset, y_offset)
        }
    }

    pub fn from_glyph(glyph: &Glyph, is_overlay: bool) -> Self {
        Self::new(is_overlay, glyph.id, glyph.x, glyph.y)
    }

    pub fn patch(&mut self, is_variable_width: bool) {
        match *self {
            Self::Overlay(id, _, ref mut y_offset) if is_variable_width => {
                const DIAERESIS: u16 = 532;
                const _DOT_ABOVE: u16 = 533;
                const _GRAVE_ACCENT: u16 = 534;
                const _ACUTE_ACCENT: u16 = 535;
                const _DOUBLE_ACUTE_ACCENT: u16 = 536;
                const _ACUTE_ACCENT_OR_CARON: u16 = 537;
                const _CIRCUMFLEX_ACCENT: u16 = 538;
                const _CARON: u16 = 539;
                const _BREVE: u16 = 540;
                const _RING_ABOVE: u16 = 541;
                const _TILDE: u16 = 542;
                const _MACRON: u16 = 543;
                const _HOOK_ABOVE: u16 = 544;
                const _DOUBLE_GRAVE_ACCENT: u16 = 545;
                const _INVERTED_BREVE: u16 = 546;
                const _TURNED_COMMA_ABOVE: u16 = 547;
                const HORN: u16 = 548;
                const DOT_BELOW: u16 = 549;
                const _DIAERESIS_BELOW: u16 = 550;
                const COMMA_BELOW: u16 = 551;
                const _CEDILLA: u16 = 552;
                const _OGONEK: u16 = 553;

                if matches!(id, DIAERESIS..=HORN | DOT_BELOW | COMMA_BELOW) {
                    *y_offset = dbg!(*y_offset).ceil();
                }
            }
            Self::Initial(id, ref mut x_offset, ref mut y_offset) if is_variable_width => {
                const DIAERESIS: u16 = 556;
                const _DOT_ABOVE: u16 = 557;
                const _GRAVE_ACCENT: u16 = 558;
                const _ACUTE_ACCENT: u16 = 559;
                const _DOUBLE_ACUTE_ACCENT: u16 = 560;
                const _CIRCUMFLEX_ACCENT: u16 = 561;
                const _OVERLINE_OR_MACRON: u16 = 562;
                const _CARON: u16 = 563;
                const _BREVE: u16 = 564;
                const _RING_ABOVE: u16 = 565;
                const _TILDE: u16 = 566;
                const _MACRON: u16 = 567;
                const _CEDILLA: u16 = 568;
                const OGONEK: u16 = 569;

                if matches!(id, DIAERESIS..=OGONEK) {
                    *x_offset = 0.0;
                    *y_offset = 0.0;
                }
            }
            Self::Overlay(id, _, ref mut y_offset) => {
                const DIAERESIS: u16 = 787;
                const _DOT_ABOVE: u16 = 788;
                const _GRAVE_ACCENT: u16 = 789;
                const _ACUTE_ACCENT: u16 = 790;
                const _DOUBLE_ACUTE_ACCENT: u16 = 791;
                const _ACUTE_ACCENT_OR_CARON: u16 = 792;
                const _CIRCUMFLEX_ACCENT: u16 = 793;
                const _CARON_2: u16 = 794;
                const _BREVE: u16 = 795;
                const _RING_ABOVE: u16 = 796;
                const _TILDE: u16 = 797;
                const _MACRON: u16 = 798;
                const _OVERLINE: u16 = 799;
                const _HOOK_ABOVE: u16 = 800;
                const _DOUBLE_GRAVE_ACCENT: u16 = 801;
                const _INVERTED_BREVE: u16 = 802;
                const _TURNED_COMMA_ABOVE: u16 = 803;
                const HORN: u16 = 804;
                const _DOT_BELOW: u16 = 805;
                const _DIAERESIS_BELOW: u16 = 806;
                const _COMMA_BELOW: u16 = 807;
                const _CEDILLA: u16 = 808;
                const _OGONEK: u16 = 809;

                if matches!(id, DIAERESIS..=HORN) {
                    *y_offset = dbg!(*y_offset).floor();
                }
            }
            Self::Initial(id, ref mut x_offset, ref mut y_offset) => {
                const DIAERESIS: u16 = 812;
                const _DOT_ABOVE: u16 = 813;
                const _GRAVE_ACCENT: u16 = 814;
                const _ACUTE_ACCENT: u16 = 815;
                const _DOUBLE_ACUTE_ACCENT: u16 = 816;
                const _CIRCUMFLEX_ACCENT: u16 = 817;
                const _OVERLINE_OR_MACRON: u16 = 818;
                const _CARON: u16 = 819;
                const _BREVE: u16 = 820;
                const _RING_ABOVE: u16 = 821;
                const _TILDE: u16 = 822;
                const _MACRON: u16 = 823;
                const _CEDILLA: u16 = 824;
                const OGONEK: u16 = 825;

                if matches!(id, DIAERESIS..=OGONEK) {
                    *x_offset = 0.0;
                    *y_offset = 0.0;
                }
            }
        }
    }
}
