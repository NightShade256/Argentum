use std::{path::PathBuf, time::Duration};

use argentum::{Argentum, ArgentumKey};
use clap::Clap;
use pixels::{PixelsBuilder, SurfaceTexture};
use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::windows::WindowBuilderExtWindows;
use winit::window::WindowBuilder;

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

fn handle_keyboard_input(argentum: &mut Argentum, input: KeyboardInput) {
    let key = match input.virtual_keycode {
        Some(VirtualKeyCode::W) => Some(ArgentumKey::Up),
        Some(VirtualKeyCode::A) => Some(ArgentumKey::Left),
        Some(VirtualKeyCode::S) => Some(ArgentumKey::Down),
        Some(VirtualKeyCode::D) => Some(ArgentumKey::Right),
        Some(VirtualKeyCode::Z) => Some(ArgentumKey::ButtonA),
        Some(VirtualKeyCode::X) => Some(ArgentumKey::ButtonB),
        Some(VirtualKeyCode::Return) => Some(ArgentumKey::Start),
        Some(VirtualKeyCode::Space) => Some(ArgentumKey::Select),

        _ => None,
    };

    if let Some(key) = key {
        if input.state == ElementState::Pressed {
            argentum.key_down(key);
        } else {
            argentum.key_up(key);
        }
    }
}

fn main() {
    // Parse CLI options
    let opt: Opt = Opt::parse();

    let (_stream, stream_handle) =
        OutputStream::try_default().expect("failed to create audio stream");

    let sink = Sink::try_new(&stream_handle).expect("failed to create audio sink");
    let event_loop = EventLoop::new();

    // Create a window
    let window = {
        let size = LogicalSize::new(480, 432);

        WindowBuilder::new()
            .with_title("Argentum")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_drag_and_drop(false)
            .build(&event_loop)
            .unwrap()
    };

    // Create a Pixels instance
    let mut pixels = {
        let window_size = window.inner_size();
        let texture = SurfaceTexture::new(window_size.width, window_size.height, &window);

        PixelsBuilder::new(160, 144, texture)
            .enable_vsync(false)
            .build()
            .expect("failed to create a Pixels instance")
    };

    // Read the ROM file
    let mut rom_path = opt.rom_file;
    let rom = std::fs::read(&rom_path).expect("failed to read the ROM file");

    // Check if there is a save file with the ROM file
    rom_path.set_extension("sav");
    let save_file = std::fs::read(&rom_path).ok();

    // Create an Argentum instance
    let mut argentum = Argentum::new(
        &rom,
        Box::new(move |buffer| {
            while sink.len() > 2 {
                std::thread::sleep(Duration::from_millis(1));
            }

            sink.append(SamplesBuffer::new(2, 48000, buffer));
        }),
        save_file,
    );

    // Run the main event loop
    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            argentum.execute_frame();
            window.request_redraw();
        }

        Event::RedrawRequested(_) => {
            pixels
                .get_frame()
                .copy_from_slice(argentum.get_framebuffer());

            pixels.render().expect("failed to render framebuffer");
        }

        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }

            WindowEvent::KeyboardInput { input, .. } => {
                handle_keyboard_input(&mut argentum, input);
            }

            WindowEvent::Resized(window_size) => {
                pixels.resize_surface(window_size.width, window_size.height);
            }

            _ => {}
        },

        Event::LoopDestroyed => {
            if let Some(ram_save) = argentum.get_ram_dump() {
                std::fs::write(&rom_path, &ram_save).expect("Failed to write save file.");
            }
        }

        _ => {}
    });
}
