use std::slice;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use softbuffer::Buffer;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RgbBufferError {
    #[error("Invalid size: ({0}, {1})")]
    InvalidSize(usize, usize),
    #[error("Invalid (x, y) coordinates: ({0}, {1})")]
    InvalidPosition(usize, usize),
}

pub type Color = [u8; 3];

pub struct RgbBuffer<'s, const X: usize, const Y: usize, D: HasDisplayHandle, W: HasWindowHandle> {
    /// The "raw" softbuffer buffer.
    pub buffer: Buffer<'s, D, W>,
    /// The "raw" RGB pixel data as a 3D array where the axes are: `Y`, `X`, and 4 (XRGB).
    /// Note that the order is: `Y, X`. Therefore, to get the pixel at `x=4, y=5`: `self.pixels[y][x]`.
    /// The color has four elements. The first element should always be 0, and the other three are R, G, and B.
    pub pixels: &'s mut [[[u8; 4]; X]],
}

impl<'s, const X: usize, const Y: usize, D: HasDisplayHandle, W: HasWindowHandle>
    RgbBuffer<'s, X, Y, D, W>
{
    /// Convert a `Buffer` into an `RgbBuffer`. This consumes `buffer` and returns an `RgbBuffer`.
    /// This returns an `Err` if `X * Y != buffer.len()` (i.e. if the dimensions of the `RgbBuffer` are incorrect).
    pub fn from_softbuffer(mut buffer: Buffer<'s, D, W>) -> Result<Self, RgbBufferError> {
        // Test whether the dimensions are valid.
        if X * Y != buffer.len() {
            Err(RgbBufferError::InvalidSize(X, Y))
        } else {
            // Convert the raw buffer to an array of rows.
            let ptr = buffer.as_mut_ptr() as *mut [[u8; 4]; X];
            // Get the 3D pixel array.
            let pixels = unsafe { slice::from_raw_parts_mut(ptr, Y) };
            Ok(RgbBuffer { buffer, pixels })
        }
    }

    /// Set the color of a single pixel.
    /// `color` is the `[r, g, b]` color.
    ///
    /// Returns an `Err` if `(x, y)` is out of bounds.
    pub fn set_pixel(&mut self, x: usize, y: usize, color: &Color) -> Result<(), RgbBufferError> {
        if !Self::is_valid_position(x, y) {
            Err(RgbBufferError::InvalidPosition(x, y))
        } else {
            self.set_pixel_unchecked(x, y, color);
            Ok(())
        }
    }

    /// Set the color of multiple pixels.
    ///
    /// - `positions`: A slice of `(x, y)` positions.
    /// - `color`: The `[r, g, b]` color.
    ///
    /// Returns an `Err` if any position in `positions` is out of bounds.
    pub fn set_pixels(
        &mut self,
        positions: &[(usize, usize)],
        color: &Color,
    ) -> Result<(), RgbBufferError> {
        // Check the positions.
        for &(x, y) in positions {
            if !Self::is_valid_position(x, y) {
                return Err(RgbBufferError::InvalidPosition(x, y));
            }
        }
        self.set_pixels_unchecked(positions, color);
        Ok(())
    }

    /// Fill a rectangle with a color.
    /// 
    /// - `x` and `y` are the coordinates of the top-left pixel.
    /// - `w` and `h` are the width and height of the rectangle.
    /// - `color` is the `[r, g, b]` color.
    /// 
    /// Returns an `Err` if the top-left or bottom-right positions are out of bounds.
    pub fn fill_rectangle(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        color: &Color,
    ) -> Result<(), RgbBufferError> {
        if !Self::is_valid_position(x, y) {
            Err(RgbBufferError::InvalidPosition(x, y))
        } else if !Self::is_valid_position(x + w, y + h) {
            Err(RgbBufferError::InvalidPosition(x + w, y + h))
        } else {
            self.fill_rectangle_unchecked(x, y, w, h, color);
            Ok(())
        }
    }

    /// Fill the buffer with an `[r, g, b]` color.
    pub fn fill(&mut self, color: &Color) {
        self.buffer
            .fill(u32::from_le_bytes([0, color[0], color[1], color[2]]));
    }

    /// Set the color of a single pixel.
    /// `color` is the `[r, g, b]` color.
    ///
    /// Panics if `(x, y)` is out of bounds.
    pub fn set_pixel_unchecked(&mut self, x: usize, y: usize, color: &Color) {
        self.pixels[y][x][1..4].copy_from_slice(color);
    }

    /// Set the color of multiple pixels.
    ///
    /// - `positions`: A slice of `(x, y)` positions.
    /// - `color`: The `[r, g, b]` color.
    ///
    /// Panics if any position in `positions` is out of bounds.
    pub fn set_pixels_unchecked(&mut self, positions: &[(usize, usize)], color: &Color) {
        // Convert the color to a softbuffer value.
        let mut rgb = [0; 4];
        rgb[1..4].copy_from_slice(color);
        // Copy the color into each position.
        for position in positions {
            self.pixels[position.1][position.0] = rgb;
        }
    }

    /// Fill a rectangle with a color.
    /// 
    /// - `x` and `y` are the coordinates of the top-left pixel.
    /// - `w` and `h` are the width and height of the rectangle.
    /// - `color` is the `[r, g, b]` color.
    /// 
    /// Panics if the top-left or bottom-right positions are out of bounds.
    pub fn fill_rectangle_unchecked(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        color: &Color,
    ) {
        // Convert the color to a softbuffer value.
        let mut rgb = [0; 4];
        rgb[1..4].copy_from_slice(color);
        // Create a row of colors and get a slice of it.
        let rgbs = &[rgb; Y][x..x + w];
        // Fill the rectangle.
        self.pixels[y..y + h]
            .iter_mut()
            .for_each(|row| row[x..x + w].copy_from_slice(rgbs));
    }

    fn is_valid_position(x: usize, y: usize) -> bool {
        x < X && y < Y
    }
}
