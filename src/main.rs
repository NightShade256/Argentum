use std::{env, path::PathBuf};

use argentum_core::{GameBoy, GbKey};
use pixels::{Pixels, SurfaceTexture};
use structopt::StructOpt;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod fps_limiter;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(StructOpt)]
#[structopt(name = "Argentum GB")]
#[structopt(version = PKG_VERSION, about = "A simple Game Boy (DMG) emulator.")]
struct Opt {
    /// The Game Boy ROM file to execute.
    #[structopt(parse(from_os_str))]
    rom_file: PathBuf,

    /// Turn on basic logging support.
    #[structopt(short, long)]
    logging: bool,
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

/// Handle the keyboard input.
fn handle_input(gb: &mut GameBoy, input: &KeyboardInput) {
    if let KeyboardInput {
        virtual_keycode: Some(keycode),
        state,
        ..
    } = input
    {
        let key = match keycode {
            VirtualKeyCode::W => Some(GbKey::UP),
            VirtualKeyCode::A => Some(GbKey::LEFT),
            VirtualKeyCode::S => Some(GbKey::DOWN),
            VirtualKeyCode::D => Some(GbKey::RIGHT),
            VirtualKeyCode::Return => Some(GbKey::START),
            VirtualKeyCode::Space => Some(GbKey::SELECT),
            VirtualKeyCode::Z => Some(GbKey::BUTTON_A),
            VirtualKeyCode::X => Some(GbKey::BUTTON_B),
            _ => None,
        };

        if let Some(key) = key {
            if *state == ElementState::Pressed {
                gb.key_down(key);
            } else {
                gb.key_up(key);
            }
        }
    }
}

/// Start running the emulator.
pub fn main() {
    // Parse command line arguments.
    let opts: Opt = Opt::from_args();
    let rom_file = opts.rom_file;

    // Setup logging.
    if opts.logging {
        env_logger::builder()
            .target(env_logger::Target::Stdout)
            .filter_module("argentum_core", log::LevelFilter::Info)
            .init();
    }

    // Read the ROM file into memory.
    let rom = std::fs::read(rom_file).expect("Failed to read the ROM file.");

    // Create a Game Boy instance and skip the bootrom.
    let mut argentum = GameBoy::new(&rom);
    argentum.skip_bootrom();

    // Create a event loop, and initialize the window and Pixels.
    let event_loop = EventLoop::new();
    let window = initialize_window(&event_loop);
    let mut pixels = initialize_pixels(&window);

    let mut fps_limiter = fps_limiter::FpsLimiter::new();

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            // Record the time of the frame.
            fps_limiter.update();

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
        }

        Event::RedrawEventsCleared => fps_limiter.limit(),

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

        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => handle_input(&mut argentum, &input),

        _ => {}
    });
}
