use std::{path::PathBuf, time::Duration};

use argentum::{Argentum, ArgentumKey};
use clap::Clap;
use pixels::{PixelsBuilder, SurfaceTexture};
use sdl2::{
    audio::{AudioQueue, AudioSpecDesired},
    event::{Event, WindowEvent},
    keyboard::Scancode,
};

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clap)]
#[clap(name = "Argentum")]
#[clap(version = PKG_VERSION, about = "A Game Boy Color emulator written in Rust.")]
struct Opt {
    #[clap(parse(from_os_str))]
    rom_file: PathBuf,

    #[clap(short, long)]
    skip_bootrom: bool,
}

/// Map a SDL_Scancode to an Argentum Key.
fn map_scancode_key(code: Scancode) -> Option<ArgentumKey> {
    match code {
        Scancode::W => Some(ArgentumKey::Up),
        Scancode::A => Some(ArgentumKey::Left),
        Scancode::S => Some(ArgentumKey::Down),
        Scancode::D => Some(ArgentumKey::Right),
        Scancode::Z => Some(ArgentumKey::ButtonA),
        Scancode::X => Some(ArgentumKey::ButtonB),
        Scancode::Return => Some(ArgentumKey::Start),
        Scancode::Space => Some(ArgentumKey::Select),

        _ => None,
    }
}

fn main() {
    // Parse CLI options, and initialize SDL
    let opt: Opt = Opt::parse();
    let sdl = sdl2::init().expect("failed to initialize SDL");

    // Initialize audio and video SDL subsystems
    let audio_subsystem = sdl
        .audio()
        .expect("failed to initialize SDL audio subsystem");

    let video_subsystem = sdl
        .video()
        .expect("failed to initialize SDL video subsystem");

    // Create a SDL window
    let window = video_subsystem
        .window("Argentum", 480, 432)
        .position_centered()
        .resizable()
        .build()
        .expect("failed to create a window");

    // Create a Pixels instance for rendering
    let mut pixels = {
        let window_size = window.drawable_size();
        let texture = SurfaceTexture::new(window_size.0, window_size.1, &window);

        PixelsBuilder::new(160, 144, texture)
            .enable_vsync(false)
            .build()
            .expect("failed to create a Pixels instance")
    };

    // Create an audio queue
    let desired_spec = AudioSpecDesired {
        freq: Some(48000),
        channels: Some(2),
        samples: Some(1024),
    };

    let audio_queue: AudioQueue<f32> = audio_subsystem
        .open_queue(None, &desired_spec)
        .expect("failed to create audio queue");

    audio_queue.resume();

    // Read the ROM file provided by the user
    let mut rom_path = opt.rom_file;
    let rom = std::fs::read(&rom_path).expect("failed to read the ROM file");

    // Check if there is a save file accompanying the ROM file, and read it
    rom_path.set_extension("sav");
    let save_file = std::fs::read(&rom_path).ok();

    // Create an Argentum instance
    let mut argentum = Argentum::new(
        &rom,
        Box::new(move |buffer| {
            while audio_queue.size() > 1024 * 4 * 2 {
                std::thread::sleep(Duration::from_millis(1));
            }

            audio_queue.queue(buffer);
        }),
        save_file,
    );

    // Create an event pump for window events
    let mut event_pump = sdl.event_pump().unwrap();

    'main: loop {
        // Handle window events if any
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    scancode: Some(code),
                    ..
                } => {
                    if let Some(key) = map_scancode_key(code) {
                        argentum.key_down(key);
                    }
                }

                Event::KeyUp {
                    scancode: Some(code),
                    ..
                } => {
                    if let Some(key) = map_scancode_key(code) {
                        argentum.key_up(key);
                    }
                }

                Event::Quit { .. } => {
                    break 'main;
                }

                Event::Window {
                    win_event: WindowEvent::Resized(width, height),
                    ..
                } => {
                    pixels.resize_surface(width as u32, height as u32);
                }

                _ => {}
            }
        }

        // Execute a frames worth of instructions
        argentum.execute_frame();

        // Update the pixels framebuffer
        pixels
            .get_frame()
            .copy_from_slice(argentum.get_framebuffer());

        // Render the framebuffer to the screen
        pixels.render().expect("failed to render framebuffer");
    }

    // Save RAM dump
    if let Some(ram_save) = argentum.get_ram_dump() {
        std::fs::write(&rom_path, &ram_save).expect("failed to write save file");
    }
}
