use core::iter;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, OriginDimensions, Size};
use embedded_graphics::image::ImageDrawable;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::transform::Transform;

use crate::image::Colors;

/// Image drawable with a reference to an area of another image drawable.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SubImage<'a, T> {
    parent: &'a T,
    area: Rectangle,
}

/// Image drawable creation, returning a sub-image.
///
/// An image drawable that implements this trait can return an image drawable with a reference to
/// an area of itself.
pub trait ImageDrawableExt: Sized {
    /// Returns the image drawable's sub-image with the specified area.   
    fn sub_image(&self, area: &Rectangle) -> SubImage<'_, Self>;
}

impl<T: ImageDrawable> ImageDrawableExt for T {
    fn sub_image(&self, area: &Rectangle) -> SubImage<'_, T> {
        let area = self.bounding_box().intersection(area);

        SubImage::new(self, area)
    }
}

impl<'a, T> SubImage<'a, T> {
    /// Creates a new image drawable with the specified parent and sub-image area.
    const fn new(parent: &'a T, area: Rectangle) -> Self {
        Self { parent, area }
    }
}

impl<T> OriginDimensions for SubImage<'_, T> {
    fn size(&self) -> Size {
        self.area.size
    }
}

impl<T: ImageDrawable> ImageDrawable for SubImage<'_, T> {
    type Color = T::Color;

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.parent.draw_sub_image(target, &self.area)
    }

    fn draw_sub_image<D>(&self, target: &mut D, area: &Rectangle) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let area = area.translate(self.area.top_left);

        self.parent.draw_sub_image(target, &area)
    }
}

impl<T: ImageDrawable + Colors<T::Color>> Colors<T::Color> for SubImage<'_, T> {
    fn colors(&self) -> impl IntoIterator<Item = T::Color> {
        let width = self.parent.size().width;
        let left = self.area.top_left.x.try_into().unwrap_or_default();
        let top = self.area.top_left.y.try_into().unwrap_or_default();
        let mut colors = self.parent.colors().into_iter();
        let mut skip = width.saturating_mul(top).saturating_add(left);
        while skip > 0 {
            skip -= 1;
            colors.next();
        }

        let mut x = 0;
        iter::from_fn(move || {
            if x == self.area.size.width {
                while x < width {
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
