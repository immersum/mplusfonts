//! Images and image drawables.
//!
//! Image drawables are wrappers around raw data or other image drawables. An image is an
//! aggregation of an image drawable and its offset from the origin. Besides the fact that the
//! image drawable is owned by the image, some of the image drawables in this module expose the
//! contiguous stream of pixel colors, allowing for data manipulation on the image level rather
//! than at the draw target only.
mod mix;
mod raw;
mod set;
mod sub;

use embedded_graphics::Drawable;
use embedded_graphics::draw_target::{DrawTarget, DrawTargetExt};
use embedded_graphics::geometry::{Dimensions, Point};
use embedded_graphics::image::ImageDrawable;
use embedded_graphics::iterator::raw::RawDataSlice;
use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::pixelcolor::raw::BigEndian;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::transform::Transform;

pub use mix::{ImageMix, Mixed};
pub use raw::ImageRaw;
pub use set::ImageSet;
pub use sub::{ImageDrawableExt, SubImage};

/// Image that owns the image drawable and holds image offset data.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Image<T: ImageDrawable> {
    image_drawable: T,
    offset: Point,
}

/// Iteration by pixels; colors only.
pub trait Colors<T> {
    /// Returns a data structure that can be turned into an iterator that yields colors of type `T`.
    fn colors(&self) -> impl IntoIterator<Item = T>;
}

impl<'a, C> Image<ImageRaw<'a, C>>
where
    C: PixelColor + From<C::Raw>,
    RawDataSlice<'a, C::Raw, BigEndian>: IntoIterator<Item = C::Raw>,
{
    /// The zero-sized, zero-offset image.
    pub const NULL: Self = Self {
        image_drawable: ImageRaw::new(&[], 0),
        offset: Point::zero(),
    };
}

impl<T: ImageDrawable> Image<T> {
    /// Creates a new image with the specified image drawable and image offset.
    pub const fn new(image_drawable: T, offset: Point) -> Self {
        Self {
            image_drawable,
            offset,
        }
    }

    /// Returns an image that is a clipped area of the same image data.
    pub fn clipped(&self, area: &Rectangle) -> Image<SubImage<'_, T>> {
        let x = self.offset.x.saturating_neg();
        let y = self.offset.y.saturating_neg();
        let offset = Point::new(x, y);
        let area = area.translate(offset);
        let area = self.image_drawable.bounding_box().intersection(&area);
        let image_drawable = self.image_drawable.sub_image(&area);

        Image::new(image_drawable, area.translate(self.offset).top_left)
    }
}

impl<T: ImageDrawable + Clone> Image<T> {
    /// Returns an image with its offset translated.
    pub fn add_offset(&self, x: i32, y: i32) -> Self {
        let x = self.offset.x.saturating_add(x);
        let y = self.offset.y.saturating_add(y);
        let offset = Point::new(x, y);

        Self::new(self.image_drawable.clone(), offset)
    }

    /// Returns an image with its offset scaled.
    pub fn mul_offset(&self, x: i32, y: i32) -> Self {
        let x = self.offset.x.saturating_mul(x);
        let y = self.offset.y.saturating_mul(y);
        let offset = Point::new(x, y);

        Self::new(self.image_drawable.clone(), offset)
    }
}

impl<T: ImageDrawable + Colors<T::Color>> Colors<T::Color> for Image<T> {
    fn colors(&self) -> impl IntoIterator<Item = T::Color> {
        self.image_drawable.colors()
    }
}

impl<T: ImageDrawable> Dimensions for Image<T> {
    fn bounding_box(&self) -> Rectangle {
        self.image_drawable.bounding_box().translate(self.offset)
    }
}

impl<T: ImageDrawable> Drawable for Image<T> {
    type Color = T::Color;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let mut target = target.translated(self.offset);

        self.image_drawable.draw(&mut target)
    }
}
