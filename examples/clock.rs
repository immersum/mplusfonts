use std::convert::Infallible;
use std::thread;
use std::time::Duration;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Alignment, Baseline, Text, TextStyleBuilder};
use embedded_graphics_simulator::SimulatorEvent::Quit;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use mplusfonts::mplus;
use mplusfonts::style::BitmapFontStyleBuilder;

/// Displays a clock, rendering the digits in the hours, the minutes, and the seconds.
pub fn main() -> Result<(), Infallible> {
    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(240, 240));

    let bitmap_font = mplus!(1, 500, 50, true, 1, 4, '0'..='9', [":"]);

    let character_style = BitmapFontStyleBuilder::new()
        .text_color(Rgb565::new(30, 60, 30))
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

    'running: loop {
        let digits = chrono::Local::now().format("%H:%M:%S").to_string();

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
    }
}
