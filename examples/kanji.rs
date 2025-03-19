use core::convert::Infallible;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use embedded_text::TextBox;
use embedded_text::style::TextBoxStyle;
use mplusfonts::mplus;
use mplusfonts::style::BitmapFontStyleBuilder;

/// Displays a sample of Japanese text using [`mplusfonts`].
#[mplusfonts::strings]
pub fn main() -> Result<(), Infallible> {
    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(240, 240));

    let bitmap_fonts = [
        #[strings::emit]
        mplus!(1, 420, cap_height(10), true, 1, 4),
        #[strings::emit]
        mplus!(2, 420, cap_height(10), true, 1, 4),
        #[strings::emit]
        mplus!(code(125), 420, cap_height(10), true, 1, 4),
    ];

    let builder = BitmapFontStyleBuilder::new().text_color(Rgb565::new(30, 60, 30));

    for (font, y) in bitmap_fonts.iter().zip([3, 80, 157]) {
        let textbox = TextBox::with_textbox_style(
            "mplusfonts（エムプラスフォンツ）は、森下浩司によって\
            デザインされているゴシック体の日本語フォントである。",
            Rectangle::new(Point::new(10, y), Size::new(220, 80)),
            builder.clone().font(font).build(),
            TextBoxStyle::default(),
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
