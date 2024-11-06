/*
The following code was based on  https://github.com/inazarenko/ssd1331-async/tree/b3006ad955dfc83100527a56d7c06d775ac3bb69
with some minor changes.
*/

use embedded_graphics_core::{
    pixelcolor::raw::ToBytes,
    prelude::{DrawTarget, OriginDimensions, PixelColor, Size},
    Pixel,
};
use log::*;

/// Memory buffer that can serve as a [`DrawTarget`].
///
/// Compared to the one in `embedded-graphics`, this one allows to use the
/// same slice of bytes to draw display areas of different shape or color
/// depth, sequentially. Because of the dynamic shape, it's likely a bit
/// slower.
pub struct Framebuffer<'a, C> {
    size: Size,
    data: &'a mut [u8],
    _color: core::marker::PhantomData<C>,
}

impl<'a, C> Framebuffer<'a, C>
where
    C: ToBytes,
{
    const BYTES_PER_PIXEL: usize = core::mem::size_of::<C::Bytes>();

    /// Creates a framebuffer.
    ///
    /// Panics if the data slice is too small to hold the requested size.
    pub fn new(data: &'a mut [u8], size: Size) -> Self {
        let n = data.len();
        let s = Self {
            size,
            data,
            _color: core::marker::PhantomData,
        };
        assert!(n >= s.pixel_count() * Self::BYTES_PER_PIXEL);

        // clear array, Note: filling array with 0 means background color is black
        s.data.fill(0);
        s
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..self.pixel_count() * Self::BYTES_PER_PIXEL]
    }

    pub fn pixel_count(&self) -> usize {
        self.size.width as usize * self.size.height as usize
    }
}

impl<'a, C> OriginDimensions for Framebuffer<'a, C> {
    fn size(&self) -> Size {
        self.size
    }
}

impl<'a, C> DrawTarget for Framebuffer<'a, C>
where
    C: PixelColor + ToBytes,
    C::Bytes: AsRef<[u8]>,
{
    type Color = C;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for p in pixels {
            // Try to convert x,y point from i32 to usize
            let (Ok(x), Ok(y)) = (usize::try_from(p.0.x), usize::try_from(p.0.y)) else {
                continue;
            };

            // If the pixel we want to draw is outside the framebuffer size ignore it
            if x >= self.size.width as usize || y >= self.size.height as usize {
                // Use this print statement to see if the framebuffer is sized correctly.
                // you should not see any of theses print statements
                info!("x = {:?}   y = {:?}", x, y);
                continue;
            }

            // Transpose x and y
            let offset = (y * self.size.width as usize + x) * Self::BYTES_PER_PIXEL;
            //info!("x = {:?}    y = {:?}    offset = {:?}", x, y, offset);

            // Copy pixel to framebuffer memory
            self.data[offset..offset + Self::BYTES_PER_PIXEL]
                .copy_from_slice(p.1.to_be_bytes().as_ref());
        }
        Ok(())
    }
}
