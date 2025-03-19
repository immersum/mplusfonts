use core::convert::Infallible;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::LineHeight;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use embedded_text::TextBox;
use embedded_text::alignment::{HorizontalAlignment, VerticalAlignment};
use embedded_text::style::TextBoxStyleBuilder;
use mplusfonts::mplus;
use mplusfonts::style::BitmapFontStyleBuilder;

/// Displays the gojūon using hiragana and katakana characters.
pub fn main() -> Result<(), Infallible> {
    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(240, 240));

    draw_hiragana(&mut display)?;
    draw_katakana(&mut display)?;

    let output_settings = OutputSettingsBuilder::new()
        .scale(3)
        .pixel_spacing(1)
        .build();

    Window::new("Simulator", &output_settings).show_static(&display);

    Ok(())
}

#[mplusfonts::strings]
fn draw_hiragana<D: DrawTarget<Color = Rgb565>>(target: &mut D) -> Result<(), D::Error> {
    #[strings::emit]
    let bitmap_font = mplus!(1, 525, line_height(20), true, 1, 4);

    let character_style = BitmapFontStyleBuilder::new()
        .text_color(Rgb565::new(30, 60, 30))
        .font(&bitmap_font)
        .build();

    let textbox_style = TextBoxStyleBuilder::new()
        .alignment(HorizontalAlignment::Center)
        .line_height(LineHeight::Pixels(21))
        .build();

    let text_columns = [
        "あ か さ た な は ま や ら わ ん",
        "い き し ち に ひ み\n\nり ゐ",
        "う く す つ ぬ ふ む ゆ る",
        "え け せ て ね へ め\n\nれ ゑ",
        "お こ そ と の ほ も よ ろ を",
    ];

    for (text, x) in text_columns.iter().zip([20, 60, 100, 140, 180]) {
        let textbox = TextBox::with_textbox_style(
            text,
            Rectangle::new(Point::new(x, 5), Size::new(20, 230)),
            character_style.clone(),
            textbox_style,
        );

        textbox.draw(target)?;
    }

    Ok(())
}

#[mplusfonts::strings]
fn draw_katakana<D: DrawTarget<Color = Rgb565>>(target: &mut D) -> Result<(), D::Error> {
    #[strings::emit]
    let bitmap_font = mplus!(1, 420, line_height(16), true, 2, 4);

    let character_style = BitmapFontStyleBuilder::new()
        .text_color(Rgb565::new(24, 48, 24))
        .font(&bitmap_font)
        .build();

    let textbox_style = TextBoxStyleBuilder::new()
        .alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Top)
        .line_height(LineHeight::Pixels(21))
        .build();

    let text_columns = [
        "ア カ サ タ ナ ハ マ ヤ ラ ワ ン",
        "イ キ シ チ ニ ヒ ミ\n\nリ ヰ",
        "ウ ク ス ツ ヌ フ ム ユ ル",
        "エ ケ セ テ ネ ヘ メ\n\nレ ヱ",
        "オ コ ソ ト ノ ホ モ ヨ ロ ヲ",
    ];

    for (text, x) in text_columns.iter().zip([40, 80, 120, 160, 200]) {
        let textbox = TextBox::with_textbox_style(
            text,
            Rectangle::new(Point::new(x, 5), Size::new(16, 230)),
            character_style.clone(),
            textbox_style,
        );

        textbox.draw(target)?;
    }

    Ok(())
}
