use std::slice;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use softbuffer::Buffer;

pub struct RgbBuffer<'s, const X: usize, const Y: usize, D: HasDisplayHandle, W: HasWindowHandle> {
    pub buffer: Buffer<'s, D, W>,
    pub pixels: &'s mut [[[u8; 4]; Y]],
}

impl<'s, const X: usize, const Y: usize, D: HasDisplayHandle, W: HasWindowHandle>
    RgbBuffer<'s, X, Y, D, W>
{
    pub fn set_pixel_unchecked(&mut self, x: usize, y: usize, color: &[u8; 3]) {
        self.pixels[x][y][1..4].copy_from_slice(color);
    }
}

impl<'s, const X: usize, const Y: usize, D: HasDisplayHandle, W: HasWindowHandle>
    From<Buffer<'s, D, W>> for RgbBuffer<'s, X, Y, D, W>
{
    fn from(mut value: Buffer<'s, D, W>) -> Self {
        let ptr = value.as_mut_ptr() as *mut [[u8; 4]; Y];
        let pixels = unsafe { slice::from_raw_parts_mut(ptr, X) };
        Self {
            buffer: value,
            pixels,
        }
    }
}
