use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use softbuffer_rgb::RgbBuffer;

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
            const X: usize = 400;
            const Y: usize = 300;
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
            let mut rgb_buffer: RgbBuffer<X, Y, _, _> = surface.buffer_mut().unwrap().into();
            // Set a pixel.
            let x = 12;
            let y = 14;
            let r = 255;
            let g = 20;
            let b = 5;
            rgb_buffer.set_pixel_unchecked(x, y, &[r, g, b]);
            // Check that the pixel was set.
            assert_eq!(rgb_buffer.pixels[x][y], [0, r, g, b]);
            // Get the index.
            let i = x * Y + y;
            assert_eq!(i, 3614);
            // Get the value.
            assert_eq!(rgb_buffer.buffer[i], u32::from_le_bytes([0, r, g, b]));
            event_loop.exit();
        }
    }

    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}
}
