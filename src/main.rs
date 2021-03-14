use std::time::{Duration, Instant};
use std::{env, path::PathBuf};

use argentum_core::GameBoy;
use pixels::{Pixels, SurfaceTexture};
use structopt::StructOpt;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(StructOpt)]
#[structopt(name = "Argentum GB")]
#[structopt(version = PKG_VERSION, about = "A simple Game Boy (DMG) emulator.")]
struct Opt {
    /// The Game Boy ROM file to execute.
    #[structopt(parse(from_os_str))]
    rom_file: PathBuf,
}

/// Initialize a winit window for rendering with Pixels.
fn initialize_window(event_loop: &EventLoop<()>) -> Window {
    WindowBuilder::new()
        .with_decorations(true)
        .with_title("Argentum GB")
        .with_min_inner_size(LogicalSize::new(160, 144))
        .with_inner_size(LogicalSize::new(480, 432))
        .build(event_loop)
        .expect("Failed to create a window.")
}

/// Initialize Pixels instance.
fn initialize_pixels(window: &Window) -> Pixels {
    let window_size = window.inner_size();

    // Create a surface texture.
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window);

    // Create pixels instance and return it.
    Pixels::new(160, 144, surface_texture).expect("Failed to initialize Pixels framebuffer.")
}

/// Start running the emulator.
pub fn main() {
    // Parse command line arguments.
    let opts: Opt = Opt::from_args();
    let rom_file = opts.rom_file;

    // Read the ROM file into memory.
    let rom = std::fs::read(rom_file).expect("Failed to read the ROM file.");

    // Create a Game Boy instance and skip the bootrom.
    let mut argentum = GameBoy::new(&rom);
    argentum.skip_bootrom();

    // Create a event loop, and initialize the window and Pixels.
    let event_loop = EventLoop::new();
    let window = initialize_window(&event_loop);
    let mut pixels = initialize_pixels(&window);

    // Stores the time of the occurence of the last frame.
    let mut last_frame = Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            // Record the time of the frame.
            last_frame = Instant::now();

            // Request a screen redraw.
            window.request_redraw();
        }

        Event::RedrawRequested(_) => {
            // Execute one frame's worth of instructions.
            argentum.execute_frame();

            // Get the PPU's framebuffer and update Pixels' framebuffer with it.
            pixels
                .get_frame()
                .copy_from_slice(argentum.get_framebuffer());

            // Render the Pixels framebuffer onto the screen.
            pixels.render().expect("Failed to render framebuffer.");

            // Limit the FPS to roughly 59.73 FPS.
            let now = Instant::now();
            let target = last_frame + Duration::from_secs_f64(1.0 / 59.73);

            if now < target {
                std::thread::sleep(target - now);
            }
        }

        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }

        Event::WindowEvent {
            event: WindowEvent::Resized(window_size),
            ..
        } if window_size.width != 0 && window_size.height != 0 => {
            pixels.resize_surface(window_size.width, window_size.height)
        }

        _ => {}
    });
}
