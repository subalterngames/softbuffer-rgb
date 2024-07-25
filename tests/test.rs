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
            let color = [0, 255, 20, 5];
            rgb_buffer.pixels[y][x] = color;
            // Check that the pixel was set.
            let sb_color = u32::from_le_bytes(color);
            assert_eq!(rgb_buffer.buffer[index(x, y)], sb_color);
            // This is ok.
            rgb_buffer.fill_rectangle(20, 20, 200, 50, [0, 67, 200, 80]);
            // Test the fill value.
            rgb_buffer.fill(color);
            assert!(rgb_buffer.buffer.iter().all(|v| *v == sb_color));

            let x = 30;
            let y = 20;
            let w = 200;
            let h = 100;
            println!("Draw a rectangle:");
            // Test raw softbuffer.
            let mut dts = [0.0; ITS];
            for dt in dts.iter_mut() {
                let t0 = Instant::now();
                for x1 in x..x + w {
                    for y1 in y..y + h {
                        rgb_buffer.buffer[index(x1, y1)] = sb_color;
                    }
                }
                *dt = (Instant::now() - t0).as_secs_f64();
            }
            println!(
                "softbuffer: {}s",
                dts.iter().sum::<f64>() / dts.iter().len() as f64
            );
            let mut dts = [0.0; ITS];
            for dt in dts.iter_mut() {
                let t0 = Instant::now();
                rgb_buffer.fill_rectangle(x, y, w, h, color);
                *dt = (Instant::now() - t0).as_secs_f64();
            }
            println!(
                "softbuffer-rgb: {}s",
                dts.iter().sum::<f64>() / dts.iter().len() as f64
            );

            // Set per-pixel.
            println!("\nSet every pixel individually:");

            // Set with raw softbuffer.
            let t0 = Instant::now();
            for x in 0..X {
                for y in 0..Y {
                    rgb_buffer.buffer[index(x, y)] = sb_color;
                }
            }
            println!("softbuffer: {}s", (Instant::now() - t0).as_secs_f64());

            // Set by setting raw pixel data.
            let t0 = Instant::now();
            for x in 0..X {
                for y in 0..Y {
                    rgb_buffer.pixels[y][x] = color;
                }
            }
            println!("softbuffer-rgb: {}s", (Instant::now() - t0).as_secs_f64());

            // Set by `set_pixels`.
            let mut positions = vec![];
            for x in 0..X {
                for y in 0..Y {
                    positions.push((x, y));
                }
            }
            let t0 = Instant::now();
            rgb_buffer.set_pixels(&positions, color);
            println!(
                "softbuffer-rgb (set_pixels): {}s",
                (Instant::now() - t0).as_secs_f64()
            );

            // End.
            event_loop.exit();
        }
    }

    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}
}

#[inline]
fn index(x: usize, y: usize) -> usize {
    y * X + x
}
