use std::path::PathBuf;

use clap::Parser;
use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use argentum::{Argentum, ArgentumKey};

/// A Game Boy Color emulator written in Rust
#[derive(Parser)]
#[command(name = "Argentum", version, about, long_about = None)]
struct Opt {
    /// A Game Boy or Game Boy Color ROM file
    #[arg(value_name = "ROM FILE")]
    rom: PathBuf,

    /// Skip the bootrom (SameBoot) on emulator start up
    #[arg(short, long)]
    skip_bootrom: bool,
}

fn map_keycode(code: KeyCode) -> Option<ArgentumKey> {
    match code {
        KeyCode::KeyW => Some(ArgentumKey::Up),
        KeyCode::KeyA => Some(ArgentumKey::Left),
        KeyCode::KeyS => Some(ArgentumKey::Down),
        KeyCode::KeyD => Some(ArgentumKey::Right),
        KeyCode::KeyZ => Some(ArgentumKey::ButtonA),
        KeyCode::KeyX => Some(ArgentumKey::ButtonB),
        KeyCode::Enter => Some(ArgentumKey::Start),
        KeyCode::Space => Some(ArgentumKey::Select),

        _ => None,
    }
}

struct App {
    emulator: Option<Argentum>,
    pixels: Option<Pixels>,
    rom_path: PathBuf,
    stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    window: Option<Window>,
}

impl App {
    pub fn new(rom_path: PathBuf) -> Self {
        Self {
            emulator: None,
            pixels: None,
            rom_path,
            stream: None,
            stream_handle: None,
            window: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Argentum")
            .with_inner_size(PhysicalSize::new(480, 432))
            .with_max_inner_size(PhysicalSize::new(480, 432))
            .with_resizable(false);

        let window = event_loop
            .create_window(window_attributes)
            .expect("failed to create a window");

        let pixels = {
            let window_size = window.inner_size();
            let texture = SurfaceTexture::new(window_size.width, window_size.height, &window);

            PixelsBuilder::new(160, 144, texture)
                .enable_vsync(false)
                .build()
                .expect("failed to create a Pixels instance")
        };

        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        // Read the ROM file and a .sav battery save if it exists.
        let rom = std::fs::read(&self.rom_path).expect("failed to read the ROM file");
        let save_file = std::fs::read(self.rom_path.with_extension("sav")).ok();

        let argentum = Argentum::new(
            rom,
            Box::new(move |buffer| {
                while sink.len() > 2 {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }

                sink.append(SamplesBuffer::new(2, 48000, buffer));
            }),
            save_file,
        );

        self.emulator = Some(argentum);
        self.pixels = Some(pixels);
        self.stream = Some(stream);
        self.stream_handle = Some(stream_handle);
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                self.emulator.as_mut().unwrap().execute_frame();

                self.pixels
                    .as_mut()
                    .unwrap()
                    .frame_mut()
                    .copy_from_slice(self.emulator.as_mut().unwrap().get_framebuffer());

                // Render the framebuffer to the screen
                self.pixels
                    .as_mut()
                    .unwrap()
                    .render()
                    .expect("failed to render framebuffer");

                self.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::Resized(size) => {
                if size.width != 0 && size.height != 0 {
                    self.pixels
                        .as_mut()
                        .unwrap()
                        .resize_surface(size.width, size.height)
                        .unwrap();
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if let Some(key) = map_keycode(code) {
                        if event.state.is_pressed() {
                            self.emulator.as_mut().unwrap().key_down(key)
                        } else {
                            self.emulator.as_mut().unwrap().key_up(key)
                        }
                    }
                }
            }

            _ => (),
        }
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        if let Some(ram_save) = self.emulator.as_ref().unwrap().get_ram_dump() {
            std::fs::write(self.rom_path.with_extension("sav"), &ram_save)
                .expect("failed to write save file");
        }
    }
}

fn main() {
    let opt = Opt::parse();
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(opt.rom);

    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();
}
