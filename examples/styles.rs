use core::convert::Infallible;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use embedded_text::TextBox;
use embedded_text::alignment::HorizontalAlignment;
use embedded_text::style::TextBoxStyle;
use mplusfonts::mplus;
use mplusfonts::style::{BitmapFontStyle, BitmapFontStyleBuilder};

/// Displays how different styles can reuse a bitmap font, applying color and enabling decorations.
#[mplusfonts::strings]
pub fn main() -> Result<(), Infallible> {
    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(240, 240));

    #[strings::emit]
    let bitmap_font = mplus!(2, 500, line_height(35), true, 4, 4);

    let styles = [
        BitmapFontStyle::new(&bitmap_font, Rgb565::new(0, 60, 30)),
        BitmapFontStyle::new(&bitmap_font, Rgb565::new(30, 60, 0)),
        BitmapFontStyleBuilder::new().font(&bitmap_font).build(),
        BitmapFontStyleBuilder::new()
            .text_color(Rgb565::new(5, 20, 5))
            .background_color(Rgb565::new(24, 60, 16))
            .font(&bitmap_font)
            .build(),
        BitmapFontStyleBuilder::new()
            .text_color(Rgb565::new(15, 15, 60))
            .underline()
            .font(&bitmap_font)
            .build(),
        BitmapFontStyleBuilder::new()
            .text_color(Rgb565::new(24, 48, 24))
            .background_color(Rgb565::new(20, 10, 10))
            .strikethrough_with_color(Rgb565::new(30, 60, 30))
            .font(&bitmap_font)
            .build(),
    ];

    let text_samples = [
        "Blue",
        "Yellow",
        "Default",
        "Inverted",
        "Underlined",
        "Struck-out",
    ];

    for ((text, y), character_style) in text_samples.iter().zip((5..).step_by(39)).zip(styles) {
        let textbox = TextBox::with_textbox_style(
            text,
            Rectangle::new(Point::new(5, y), Size::new(230, 40)),
            character_style,
            TextBoxStyle::with_alignment(HorizontalAlignment::Center),
        );

        textbox.draw(&mut display)?;
    }

    let output_settings = OutputSettingsBuilder::new()
        .scale(3)
        .pixel_spacing(1)
        .build();

    #[strings::skip]
    Window::new("Simulator", &output_settings).show_static(&display);

    Ok(())
}
