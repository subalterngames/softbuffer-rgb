use std::slice;
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
const RANDOM_COLORS: &[u8; X * Y * 3] = include_bytes!("colors");
const HELLO_WORLD: &[u8; 87528] = include_bytes!("hello_world");
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
            assert_eq!(rgb_buffer.pixels[y][x], [0, r, g, b]);
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

            // Set random colors for every pixel.
            println!("Set every pixel to a random color:");

            // Convert `COLORS` into a 3D RGB array.
            let ptr = RANDOM_COLORS.as_ptr() as *const [[u8; 3]; X];
            let rgb3 = unsafe { slice::from_raw_parts(ptr, Y) };

            // Convert `COLORS` into u32s.
            let mut u32s = [[0; Y]; X];
            for (x, row) in rgb3.iter().enumerate() {
                for (y, color) in row.iter().enumerate() {
                    u32s[y][x] = u32::from_le_bytes([0, color[0], color[1], color[2]]);
                }
            }

            // Raw softbuffer.
            let mut t0 = Instant::now();
            for x in 0..X {
                for y in 0..Y {
                    rgb_buffer.buffer[index(x, y)] = u32s[x][y];
                }
            }
            println!("softbuffer: {}s", (Instant::now() - t0).as_secs_f64());

            // Raw pixels.
            // Convert `COLORS` into a 4D XRGB array.
            let mut rgb4 = [[[0u8; 4]; Y]; X];
            for (x, row) in rgb3.iter().enumerate() {
                for (y, color) in row.iter().enumerate() {
                    rgb4[y][x] = [0, color[0], color[1], color[2]];
                }
            }
            t0 = Instant::now();
            for x in 0..X {
                for y in 0..Y {
                    rgb_buffer.pixels[y][x] = rgb4[x][y];
                }
            }
            println!(
                "softbuffer-rgb (`self.pixel`): {}s",
                (Instant::now() - t0).as_secs_f64()
            );

            // `set_pixel`
            t0 = Instant::now();
            for x in 0..X {
                for y in 0..Y {
                    let _ = rgb_buffer.set_pixel(x, y, &rgb3[y][x]);
                }
            }
            println!(
                "softbuffer-rgb (`self.set_pixel(x, y, color)`): {}s",
                (Instant::now() - t0).as_secs_f64()
            );

            // `set_pixel_unchecked`
            t0 = Instant::now();
            for x in 0..X {
                for y in 0..Y {
                    rgb_buffer.set_pixel_unchecked(x, y, &rgb3[y][x]);
                }
            }
            println!(
                "softbuffer-rgb (`self.set_pixel_unchecked(x, y, color)`): {}s",
                (Instant::now() - t0).as_secs_f64()
            );

            // Blit "Hello World!" to the buffer.
            // Convert raw bytes to values.
            let mut positions = vec![];
            for raw_position in HELLO_WORLD.chunks_exact(4) {
                positions.push((
                    u16::from_le_bytes([raw_position[0], raw_position[1]]) as usize,
                    u16::from_le_bytes([raw_position[2], raw_position[3]]) as usize,
                ));
            }

            println!("");
            println!("Set multiple pixels of the same color:");

            // Softbuffer.
            t0 = Instant::now();
            for position in positions.iter() {
                rgb_buffer.buffer[index(position.0, position.1)] = 0;
            }
            println!("softbuffer: {}s", (Instant::now() - t0).as_secs_f64());

            // `set_pixels`
            let hello_world_color = [0, 0, 0];
            t0 = Instant::now();
            let _ = rgb_buffer.set_pixels(&positions, &hello_world_color);
            println!(
                "softbuffer-rbg (set_pixels): {}s",
                (Instant::now() - t0).as_secs_f64()
            );

            // `set_pixels_unchecked`
            t0 = Instant::now();
            let _ = rgb_buffer.set_pixels_unchecked(&positions, &hello_world_color);
            println!(
                "softbuffer-rbg (set_pixels_unchecked): {}s",
                (Instant::now() - t0).as_secs_f64()
            );

            let x = 30;
            let y = 20;
            let w = 200;
            let h = 100;
            println!("");
            println!("Draw a rectangle:");
            // Test raw softbuffer.
            let sb_color = u32::from_le_bytes([0, r, g, b]);
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
                rgb_buffer.fill_rectangle_unchecked(x, y, w, h, &[r, g, b]);
                *dt = (Instant::now() - t0).as_secs_f64();
            }
            println!(
                "softbuffer-rgb: {}s",
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
    y * X + x
}
