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

/// Displays sentences made of English words, using variable-width and monospaced fonts.
#[mplusfonts::strings]
pub fn main() -> Result<(), Infallible> {
    use HorizontalAlignment::*;

    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(240, 240));

    let bitmap_fonts = [
        #[strings::emit]
        mplus!(1, 480, x_height(7), true, 4, 4),
        #[strings::emit]
        mplus!(2, 480, x_height(7), true, 4, 4),
        #[strings::emit]
        mplus!(code(125), 480, x_height(7), true, 4, 4),
    ];

    let text_alignments = [Justified, Justified, Left];

    let builder = BitmapFontStyleBuilder::new().text_color(Rgb565::new(30, 60, 30));

    for ((font, y), alignment) in bitmap_fonts.iter().zip([3, 80, 157]).zip(text_alignments) {
        let textbox = TextBox::with_textbox_style(
            "Sphinx of black quartz, bite the wax tadpole? Very good job! \
            Efficient deflate toffee. 1 VA is not a watt.",
            Rectangle::new(Point::new(10, y), Size::new(220, 80)),
            builder.clone().font(font).build(),
            TextBoxStyle::with_alignment(alignment),
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
