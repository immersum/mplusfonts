use std::convert::Infallible;
use std::thread;
use std::time::Duration;

use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Alignment, Baseline, Text, TextStyleBuilder};
use embedded_graphics_simulator::SimulatorEvent::Quit;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use mplusfonts::mplus;
use mplusfonts::style::BitmapFontStyleBuilder;

/// Displays a counter that renders values `0x00` through `0xFF`, looping over and over.
pub fn main() -> Result<(), Infallible> {
    let mut display: SimulatorDisplay<Rgb888> = SimulatorDisplay::new(Size::new(240, 240));

    let bitmap_font = mplus!(code(100), 500, 50, false, 1, 8, '0'..='9', 'A'..='F', ["x"]);

    let character_style = BitmapFontStyleBuilder::new()
        .text_color(Rgb888::new(240, 240, 240))
        .font(&bitmap_font)
        .build();

    let text_style = TextStyleBuilder::new()
        .alignment(Alignment::Center)
        .baseline(Baseline::Middle)
        .build();

    let output_settings = OutputSettingsBuilder::new()
        .scale(3)
        .pixel_spacing(1)
        .build();

    let mut window = Window::new("Simulator", &output_settings);

    let mut ticks: u8 = 0;

    'running: loop {
        let digits = format!("0x{ticks:02X}");

        let text = Text::with_text_style(
            &digits,
            Point::new(120, 120),
            character_style.clone(),
            text_style,
        );

        text.draw(&mut display)?;

        window.update(&display);

        if window.events().any(|e| e == Quit) {
            break 'running Ok(());
        }

        thread::sleep(Duration::from_millis(50));

        ticks = ticks.wrapping_add(1);
    }
}
