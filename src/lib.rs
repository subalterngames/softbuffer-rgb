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

type Color = [u8; 3];

pub struct RgbBuffer<'s, const X: usize, const Y: usize, D: HasDisplayHandle, W: HasWindowHandle> {
    pub buffer: Buffer<'s, D, W>,
    pub pixels: &'s mut [[[u8; 4]; X]],
}

impl<'s, const X: usize, const Y: usize, D: HasDisplayHandle, W: HasWindowHandle>
    RgbBuffer<'s, X, Y, D, W>
{
    pub fn from_softbuffer(mut value: Buffer<'s, D, W>) -> Result<Self, RgbBufferError> {
        // Test whether the dimensions are valid.
        if X * Y != value.len() {
            Err(RgbBufferError::InvalidSize(X, Y))
        } else {
            // Convert the raw buffer to an array of rows.
            let ptr = value.as_mut_ptr() as *mut [[u8; 4]; X];
            // Get the 3D pixel array.
            let pixels = unsafe { slice::from_raw_parts_mut(ptr, Y) };
            Ok(RgbBuffer {
                buffer: value,
                pixels,
            })
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: &Color) -> Result<(), RgbBufferError> {
        if !Self::is_valid_position(x, y) {
            Err(RgbBufferError::InvalidPosition(x, y))
        } else {
            self.set_pixel_unchecked(x, y, color);
            Ok(())
        }
    }

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

    pub fn fill(&mut self, color: &Color) {
        self.buffer
            .fill(u32::from_le_bytes([0, color[0], color[1], color[2]]));
    }

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

    pub fn set_pixel_unchecked(&mut self, x: usize, y: usize, color: &Color) {
        self.pixels[x][y][1..4].copy_from_slice(color);
    }

    pub fn set_pixels_unchecked(&mut self, positions: &[(usize, usize)], color: &Color) {
        // Convert the color to a softbuffer value.
        let mut rgb = [0; 4];
        rgb[1..4].copy_from_slice(color);
        // Fast-copy.
        positions
            .iter()
            .for_each(|(x, y)| self.pixels[*x][*y].copy_from_slice(&rgb));
    }

    fn is_valid_position(x: usize, y: usize) -> bool {
        x < X && y < Y
    }
}
