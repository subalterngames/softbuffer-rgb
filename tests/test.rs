use std::time::Instant;

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
const ITS: usize = 10;

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
            // Set a pixel.
            let x = 12;
            let y = 14;
            let r = 255;
            let g = 20;
            let b = 5;
            rgb_buffer.set_pixel_unchecked(x, y, &[r, g, b]);
            // Check that the pixel was set.
            assert_eq!(rgb_buffer.pixels[x][y], [0, r, g, b]);
            let softbuffer_value = u32::from_le_bytes([0, r, g, b]);
            // Test the u32 value.
            assert_eq!(rgb_buffer.buffer[index(x, y)], softbuffer_value);
            // This is ok.
            assert!(rgb_buffer
                .fill_rectangle(20, 20, 200, 50, &[67, 200, 80])
                .is_ok());
            // This is not ok.
            assert!(rgb_buffer
                .fill_rectangle(20, 20, Y * 3, 50, &[67, 200, 80])
                .is_err());
            // Test the fill value.
            rgb_buffer.fill(&[r, g, b]);
            assert!(rgb_buffer.buffer.iter().all(|v| *v == softbuffer_value));

            let mut dts = [0.0; ITS];
            let x = 30;
            let y = 20;
            let w = 200;
            let h = 100;
            for dt in dts.iter_mut() {
                let t0 = Instant::now();
                rgb_buffer.fill_rectangle_unchecked(x, y, w, h, &[r, g, b]);
                *dt = (Instant::now() - t0).as_secs_f64();
            }
            println!(
                "Draw a rectangle:\nsoftbuffer-rgb: {}s",
                dts.iter().sum::<f64>() / dts.iter().len() as f64
            );
            // Test raw softbuffer.
            let color = u32::from_le_bytes([0, r, g, b]);
            let mut dts = [0.0; ITS];
            for dt in dts.iter_mut() {
                let t0 = Instant::now();
                for x1 in x..x + w {
                    for y1 in y..y + h {
                        rgb_buffer.buffer[index(x1, y1)] = color;
                    }
                }
                *dt = (Instant::now() - t0).as_secs_f64();
            }
            println!(
                "softbuffer: {}s",
                dts.iter().sum::<f64>() / dts.iter().len() as f64
            );

            // End.
            event_loop.exit();
        }
    }

    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}
}

#[inline]
fn index(x: usize, y: usize) -> usize {
    x * X + y
}
