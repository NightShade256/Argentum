mod renderer;
use std::{env, path::PathBuf};

use argentum_core::{GameBoy, GbKey};
use clap::Clap;
use glutin::{Api, ContextBuilder, GlProfile, GlRequest, dpi::LogicalSize, event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent}, event_loop::{ControlFlow, EventLoop}, platform::ContextTraitExt, window::WindowBuilder};
use renderer::Renderer;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clap)]
#[clap(name = "Argentum GB")]
#[clap(version = PKG_VERSION, about = "A simple Game Boy (DMG) emulator.")]
struct Opt {
    /// The Game Boy ROM file to execute.
    #[clap(parse(from_os_str))]
    rom_file: PathBuf,

    /// Turn on basic logging support.
    #[clap(short, long)]
    logging: bool,
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
    let opts: Opt = Opt::parse();
    let rom_file = opts.rom_file;

    // Setup logging.
    if opts.logging {
        // env_logger::builder()
        //     .target(env_logger::Target::Stdout)
        //     .filter_module("argentum_core", log::LevelFilter::Info)
        //     .init();
    }

    // Read the ROM file into memory.
    let rom = std::fs::read(rom_file).expect("Failed to read the ROM file.");

    // Create a Game Boy instance and skip the bootrom.
    let mut argentum = GameBoy::new(&rom);
    argentum.skip_bootrom();

    // Create a event loop, and initialize the window and Pixels.
    let event_loop = EventLoop::new();

    let wb = WindowBuilder::new()
        .with_decorations(true)
        .with_title("Argentum GB")
        .with_min_inner_size(LogicalSize::new(160, 144))
        .with_inner_size(LogicalSize::new(480, 432));

    let window = unsafe {
        ContextBuilder::new()
            .with_gl(GlRequest::Latest)
            .with_gl_profile(GlProfile::Core)
            .with_vsync(true)
            .build_windowed(wb, &event_loop)
            .unwrap()
            .make_current()
            .unwrap()
    };

    let mut renderer = Renderer::new(|s| window.get_proc_address(s) as *const _);

    let window_size = window.window().inner_size();
    renderer.set_viewport(window_size.width, window_size.height);

    println!("OpenGL Context: {:#?}", window.get_api());

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            // // Record the time of the frame.
            // fps_limiter.update();

            // Request a screen redraw.
            window.window().request_redraw();
        }

        Event::RedrawRequested(_) => {
            // Execute one frame's worth of instructions.
            argentum.execute_frame();

            renderer.render_buffer(argentum.get_framebuffer());
            window.swap_buffers().unwrap();
        }

        //Event::RedrawEventsCleared => fps_limiter.limit(),
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }

        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => handle_input(&mut argentum, &input),

        _ => {}
    });
}
