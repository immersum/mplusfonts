use core::convert::Infallible;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use embedded_text::TextBox;
use embedded_text::alignment::HorizontalAlignment;
use embedded_text::style::TextBoxStyle;
use mplusfonts::mplus;
use mplusfonts::style::BitmapFontStyleBuilder;

/// Displays how font-based kerning can be enabled, either by using `strings::emit` or via `kern`.
#[mplusfonts::strings]
pub fn main() -> Result<(), Infallible> {
    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(240, 240));

    #[rustfmt::skip]
    let bitmap_fonts = [
        #[strings::emit]
        mplus!(2, 575, line_height(20), true, 4, 4),

        mplus!(2, 575, line_height(20), true, 4, 4, ' '..='~', ["ff", "ffi", "ffl"]),

        mplus!(2, 575, line_height(20), true, 4, 4, kern(' '..='~', ["ff", "ffi", "ffl"])),
    ];

    let builder = BitmapFontStyleBuilder::new().text_color(Rgb565::new(30, 60, 30));

    for (font, y) in bitmap_fonts.iter().zip([3, 80, 157]) {
        let textbox = TextBox::with_textbox_style(
            "Foxberry waffle expert duo Tom & Steffi quack in Tokyo. \
            Offers go where? There's no place like ~",
            Rectangle::new(Point::new(10, y), Size::new(220, 80)),
            builder.clone().font(font).build(),
            TextBoxStyle::with_alignment(HorizontalAlignment::Justified),
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
