//! **`sotfbuffer-rgb` is a wrapper around `softbuffer` that rearranges raw buffer data as a 3D array: `(width, height, color)`.**
//! The result is much easier to work with while being just as fast as `softbuffer`, and in some cases significantly faster.
//!
//! ```rust
//!use softbuffer::{Context, Surface};
//!use std::num::NonZeroU32;
//!use winit::application::ApplicationHandler;
//!use winit::dpi::LogicalSize;
//!use winit::event::{StartCause, WindowEvent};
//!use winit::event_loop::{ActiveEventLoop, EventLoop};
//!use winit::window::{Window, WindowAttributes, WindowId};
//!
//!use softbuffer_rgb::RgbBuffer;
//!
//!const X: usize = 400;
//!const Y: usize = 300;
//!
//!fn main() {
//!    let mut app = App::default();
//!    let event_loop = EventLoop::new().unwrap();
//!    event_loop.run_app(&mut app).unwrap();
//!}
//!
//!#[derive(Default)]
//!struct App {
//!    window: Option<Window>,
//!}
//!
//!impl ApplicationHandler for App {
//!    fn resumed(&mut self, _: &ActiveEventLoop) {}
//!
//!    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
//!        if let StartCause::Init = cause {
//!            let window_attributes =
//!                WindowAttributes::default().with_inner_size(LogicalSize::new(X as u32, Y as u32));
//!            // Create the window.
//!            self.window = Some(event_loop.create_window(window_attributes).unwrap());
//!            // Get the window.
//!            let window = self.window.as_ref().unwrap();
//!            let context = Context::new(window).unwrap();
//!            let mut surface = Surface::new(&context, &window).unwrap();
//!            surface
//!                .resize(
//!                    NonZeroU32::new(X as u32).unwrap(),
//!                    NonZeroU32::new(Y as u32).unwrap(),
//!                )
//!                .unwrap();
//!            let mut rgb_buffer =
//!                RgbBuffer::<X, Y, _, _>::from_softbuffer(surface.buffer_mut().unwrap()).unwrap();
//!            rgb_buffer.set_pixel(12, 12, &[200, 100, 30]).unwrap();
//!            event_loop.exit();
//!        }
//!    }
//!
//!    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}
//!}
//!```

use std::slice;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
pub use softbuffer;
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

/// An `RgbBuffer` contains a softbuffer `buffer` and `pixels`, a mutable slice of the same data.
/// `buffer` and `pixels` reference the same underlying data.
/// Modifying the elements of one will affect the values of the other.
/// 
/// In terms of speed:
/// 
/// - Setting values in `pixels` is approximately 6 times faster than setting raw values in `buffer` because you don't need to convert (x, y) coordinates to index values.
/// - `set_pixel_unchecked(x, y, color)` is slightly slower than setting raw values in `buffer`.
/// - `set_pixels_unchecked(positions, color)` is approximately 10 times faster than setting raw values in `buffer` (assuming that you've already cached `positions`).
/// - `fill(color)` is the same speed as `buffer.fill(value)`.
/// - `fill_rectangle_unchecked(x, y, w, h, color)` is *100 times faster* than filling a rectangle in the raw `buffer`.
/// 
/// Many functions have checked and unchecked versions.
/// The checked functions will check whether all values are within the bounds of `pixels`.
/// The unchecked functions don't do this and are therefore faster.
/// 
/// In `self.pixels`, color data is represented as a 4-element array where the first element is always 0.
/// This will align the color data correctly for `softbuffer`.
/// In all functions, `color` is a 3-element array that internally is converted into a valid 4-element array.
pub struct RgbBuffer<'s, const X: usize, const Y: usize, D: HasDisplayHandle, W: HasWindowHandle> {
    /// The "raw" softbuffer `Buffer`.
    pub buffer: Buffer<'s, D, W>,
    /// The "raw" RGB pixel data as a 3D array where the axes are: `Y`, `X`, and 4 (XRGB).
    /// Note that the order is: `Y, X`. Therefore, to get the pixel at `x=4, y=5`: `self.pixels[y][x]`.
    /// The color has four elements. The first element should always be 0, and the other three are R, G, and B: `self.pixels[y][x] = [0, 200, 160, 30];`
    /// This will align the color data correctly for `softbuffer`.
    pub pixels: &'s mut [[[u8; 4]; X]],
}

impl<'s, const X: usize, const Y: usize, D: HasDisplayHandle, W: HasWindowHandle>
    RgbBuffer<'s, X, Y, D, W>
{
    /// Convert a `Buffer` into an `RgbBuffer`. This consumes `buffer` and returns an `RgbBuffer`.
    /// This returns an `Err` if `X * Y != buffer.len()` (i.e. if the dimensions of the `RgbBuffer` are invalid).
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
        self.pixels[y][x] = [0, color[0], color[1], color[2]];
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
