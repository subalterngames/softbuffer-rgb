//! **`sotfbuffer-rgb` is a wrapper around `softbuffer` that rearranges raw buffer data as a 3D array: `(width, height, color)`.**
//! The result is much easier to work with while being just as fast as `softbuffer`, and in some cases significantly faster.
//!
//! ## The Problem
//!
//! `softbuffer` stores pixel data in a u32 buffer where each u32 is an "0RGB" color.
//! The first byte is always zero, the second byte is red, the third byte is green, and the fourth byte is blue.
//!
//! Thus:
//!
//! - While we might want to describe a color as a three-element array, we must add a 0 at the start: `let color = [0, 200, 70, 10];`
//!- `softbuffer` can't use the above example. You must convert it to a u32: `let color = u32::from_le_bytes([0, 200, 70, 10])'`. Converting from a more-intuitive array to a less-intuitive u32 is an operation and there costs time.
//!
//! Additionally, `softbuffer` buffers are one-dimensional. Typically, you'll want to program in a 2D (x, y) coordinate space, meaning that you'll have to convert 2D (x, y) coordinates to 1D indices. It's a cheap operation but if you have to do it for many pixels, per frame, the performance cost can add up!
//!
//! ## The Solution
//!
//! `softbuffer-rgb` uses a tiny bit of unsafe code to rearrange the raw buffer data into a 3D array: `(width, height, 0RGB)`.
//! Modifying this `pixels` array will modify the the underlying u32 buffer array, and vice versa.
//!
//! As a result:
//!
//! - `softbuffer-rgb` can be easier to use than `softbuffer`.
//! - `softbuffer-rgb` can, in many cases, be faster, simply because you don't need to convert to u32s and you don't need to convert (x, y) coordinates to indices.
//!
//! ## The Caveat
//!
//! `softbuffer-rgb` relies on generic constants to define the size of `pixels`, meaning that the buffer size must be known at compile-time.
//!
//! ## The Example
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
//!            let x = 12;
//!            let y = 23;
//!            rgb_buffer.pixels[y][x] = [0, 200, 100, 70];
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

pub type Color = [u8; 4];

/// An `RgbBuffer` contains a softbuffer `buffer` and `pixels`, a mutable slice of the same data.
/// `buffer` and `pixels` reference the same underlying data.
/// Modifying the elements of one will affect the values of the other.
///
///
/// Color data is represented as a 4-element array where the first element is always 0.
/// This will align the color data correctly for `softbuffer`.
pub struct RgbBuffer<'s, const X: usize, const Y: usize, D: HasDisplayHandle, W: HasWindowHandle> {
    /// The "raw" softbuffer `Buffer`.
    pub buffer: Buffer<'s, D, W>,
    /// The "raw" RGB pixel data as a 3D array where the axes are: `Y`, `X`, and 4 (XRGB).
    /// Note that the order is: `Y, X`. Therefore, to get the pixel at `x=4, y=5`: `self.pixels[y][x]`.
    /// The color has four elements. The first element should always be 0, and the other three are R, G, and B: `self.pixels[y][x] = [0, 200, 160, 30];`
    /// This will align the color data correctly for `softbuffer`.
    pub pixels: &'s mut [[Color; X]],
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
            let ptr = buffer.as_mut_ptr() as *mut [Color; X];
            // Get the 3D pixel array.
            let pixels = unsafe { slice::from_raw_parts_mut(ptr, Y) };
            Ok(RgbBuffer { buffer, pixels })
        }
    }

    /// Fill the buffer with an `[0, r, g, b]` color.
    pub fn fill(&mut self, color: Color) {
        self.buffer.fill(u32::from_le_bytes(color));
    }

    /// Set the color of multiple pixels.
    ///
    /// - `positions`: A slice of `(x, y)` positions.
    /// - `color`: The `[0, r, g, b]` color.
    ///
    /// Panics if any position in `positions` is out of bounds.
    pub fn set_pixels(&mut self, positions: &[(usize, usize)], color: Color) {
        // Copy the color into each position.
        for position in positions {
            self.pixels[position.1][position.0] = color;
        }
    }

    /// Fill a rectangle with a color.
    ///
    /// - `x` and `y` are the coordinates of the top-left pixel.
    /// - `w` and `h` are the width and height of the rectangle.
    /// - `color` is the `[0, r, g, b]` color.
    ///
    /// Panics if the top-left or bottom-right positions are out of bounds.
    pub fn fill_rectangle(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        // Create a row of colors and get a slice of it.
        let colors = &[color; Y][x..x + w];
        // Fill the rectangle.
        self.pixels[y..y + h]
            .iter_mut()
            .for_each(|cols| cols[x..x + w].copy_from_slice(colors));
    }
}
