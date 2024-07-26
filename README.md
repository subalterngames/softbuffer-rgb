# softbuffer-rgb

 **`softbuffer-rgb` is a wrapper around `softbuffer` that makes it easier to modify a raw pixel buffer.**
 
 Instead of doing this:
 
 ```ignore
 buffer.buffer_mut()[y * width + x] = u32::from_le_bytes([0, 200, 70, 10]);
 ```
 
 ...you can now do this:
 
 ```ignore
 buffer.pixels[y][x] = [0, 200, 70, 10];
 ```

 ## Problem

 `softbuffer` stores pixel data in a u32 buffer where each u32 is an "0RGB" color.
 The first byte is always zero, the second byte is red, the third byte is green, and the fourth byte is blue.

 It's intuitive to store colors as arrays, like this: 
 
```rust
let color = [0, 200, 70, 10];
```
 But in `softbuffer`, colors need to be u32s:
 
```rust
let color = u32::from_le_bytes([0, 200, 70, 10]);
```

 Additionally, `softbuffer` buffers are one-dimensional. 
 Typically, you'll want to program in a 2D (x, y) coordinate space, meaning that you'll have to convert 2D (x, y) coordinates to 1D indices. 
 It's a cheap operation but if you have to do it for many pixels, per frame, the performance cost can add up!

 ## Solution

 `softbuffer-rgb` uses a tiny bit of unsafe code to rearrange the raw buffer data into a 3D array: `(width, height, 0RGB)`.
 Modifying this `pixels` array will modify the the underlying u32 buffer array, and vice versa.

 As a result:

 - `softbuffer-rgb` can be easier to use than `softbuffer`.
 - `softbuffer-rgb` can be slightly faster because you don't need to convert to u32s and you don't need to convert (x, y) coordinates to indices.

 ## Caveat

 `softbuffer-rgb` relies on generic constants to define the size of `pixels`, meaning that the buffer size must be known at compile-time.

 ## Example

 ```rust
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use softbuffer_rgb::RgbBuffer;

const X: usize = 400;
const Y: usize = 300;

fn main() {
    let mut app = App::default();
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut app).unwrap();
}

#[derive(Default)]
struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, _: &ActiveEventLoop) {}

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if let StartCause::Init = cause {
            let window_attributes =
                WindowAttributes::default().with_inner_size(LogicalSize::new(X as u32, Y as u32));
            // Create the window.
            self.window = Some(event_loop.create_window(window_attributes).unwrap());
            // Get the window.
            let window = self.window.as_ref().unwrap();
            let context = Context::new(window).unwrap();
            let mut surface = Surface::new(&context, &window).unwrap();
            surface
                .resize(
                    NonZeroU32::new(X as u32).unwrap(),
                    NonZeroU32::new(Y as u32).unwrap(),
                )
                .unwrap();
            let mut rgb_buffer =
                RgbBuffer::<X, Y, _, _>::from_softbuffer(surface.buffer_mut().unwrap()).unwrap();
            let x = 12;
            let y = 23;
            rgb_buffer.pixels[y][x] = [0, 200, 100, 70];
            event_loop.exit();
        }
    }

    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}
}
```