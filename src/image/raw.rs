use core::iter;
use core::marker::PhantomData;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{OriginDimensions, Size};
use embedded_graphics::image;
use embedded_graphics::image::ImageDrawable;
use embedded_graphics::iterator::raw::RawDataSlice;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::pixelcolor::raw::{BigEndian, RawData};
use embedded_graphics::primitives::Rectangle;

use crate::image::Colors;

/// Image drawable with a reference to raw data interpreted as colors having type `C`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ImageRaw<'a, C>
where
    C: PixelColor + From<C::Raw>,
{
    data: &'a [u8],
    size: Size,
    phantom: PhantomData<&'a C>,
}

impl<'a, C> ImageRaw<'a, C>
where
    C: PixelColor + From<C::Raw>,
{
    /// Creates a new image drawable with the specified raw data and image width.
    pub const fn new(data: &'a [u8], width: u32) -> Self {
        let bits = C::Raw::BITS_PER_PIXEL * width as usize;
        let bytes_per_row = bits / 8 + (bits % 8 > 0) as usize;
        let Some(height) = data.len().checked_div(bytes_per_row) else {
            return Self {
                data: &[],
                size: Size::zero(),
                phantom: PhantomData,
            };
        };

        Self {
            data,
            size: Size::new(width, height as u32),
            phantom: PhantomData,
        }
    }
}

impl<C> OriginDimensions for ImageRaw<'_, C>
where
    C: PixelColor + From<C::Raw>,
{
    fn size(&self) -> Size {
        self.size
    }
}

impl<'a, C> ImageDrawable for ImageRaw<'a, C>
where
    C: PixelColor + From<C::Raw>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    type Color = C;

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        image::ImageRaw::new(self.data, self.size.width).draw(target)
    }

    fn draw_sub_image<D>(&self, target: &mut D, area: &Rectangle) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        image::ImageRaw::new(self.data, self.size.width).draw_sub_image(target, area)
    }
}

impl<'a, C> Colors<C> for ImageRaw<'a, C>
where
    C: PixelColor + From<C::Raw>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    fn colors(&self) -> impl IntoIterator<Item = C> {
        let bits = C::Raw::BITS_PER_PIXEL * self.size.width as usize;
        let bytes_per_row = bits / 8 + (bits % 8 > 0) as usize;
        let x_length = bytes_per_row * 8 / C::Raw::BITS_PER_PIXEL;
        let raw_data = RawDataSlice::new(self.data);
        let mut colors = raw_data.into_iter().map(Into::into);
        let mut x = 0;
        iter::from_fn(move || {
            if x == self.size.width {
                while x < x_length as u32 {
                    x += 1;
                    colors.next();
                }

                x = 0;
            }

            x += 1;
            colors.next()
        })
    }
}
