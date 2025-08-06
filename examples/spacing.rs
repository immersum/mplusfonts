use core::convert::Infallible;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use embedded_text::TextBox;
use embedded_text::style::TextBoxStyle;
use mplusfonts::mplus;
use mplusfonts::style::BitmapFontStyleBuilder;

/// Displays how fullwidth forms are contracted or expanded with monospaced fonts in Japanese text.
#[mplusfonts::strings]
pub fn main() -> Result<(), Infallible> {
    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(240, 240));

    let bitmap_fonts = [
        #[strings::emit]
        mplus!(1, 450, 15, true, 4, 4),
        #[strings::emit]
        mplus!(code(100), 450, 15, true, 4, 4),
        #[strings::emit]
        mplus!(code(125), 450, 15, true, 4, 4),
    ];

    let builder = BitmapFontStyleBuilder::new()
        .text_color(Rgb565::new(5, 20, 5))
        .background_color(Rgb565::new(16, 60, 24));

    for (font, y) in bitmap_fonts.iter().zip([3, 80, 157]) {
        let textbox = TextBox::with_textbox_style(
            " irohanihoheto      。\n\
            以呂波耳本部止「[{}]」\n\
            いろはにほへと『<==>』",
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
