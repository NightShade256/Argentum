use std::{env, ffi::CStr, path::PathBuf};

use argentum_core::{GameBoy, GbKey};
use clap::Clap;
use glutin::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder, GlProfile, GlRequest,
};

mod renderer;
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
fn handle_keyboard_input(gb: &mut GameBoy, input: &KeyboardInput) {
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

    // Setup logging.
    if opts.logging {
        env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    }

    // Read the ROM file into memory.
    let rom = std::fs::read(opts.rom_file).expect("Failed to read the ROM file.");

    // Create a Game Boy instance and skip the bootrom.
    let mut argentum = GameBoy::new(&rom);
    argentum.skip_bootrom();

    // Create a event loop, and initialize the window and the OpenGL based renderer.
    let event_loop = EventLoop::new();

    let wb = WindowBuilder::new()
        .with_decorations(true)
        .with_resizable(false)
        .with_title("Argentum GB")
        .with_min_inner_size(LogicalSize::new(160, 144))
        .with_inner_size(LogicalSize::new(480, 432));

    let ctx = unsafe {
        ContextBuilder::new()
            .with_gl(GlRequest::Latest)
            .with_gl_profile(GlProfile::Core)
            .with_vsync(true)
            .build_windowed(wb, &event_loop)
            .unwrap()
            .make_current()
            .unwrap()
    };

    let mut renderer = Renderer::new(|s| {
        let c_str = unsafe { CStr::from_ptr(s as _) };

        ctx.get_proc_address(c_str.to_str().unwrap()) as _
    });

    // Query the window size and set GL viewport.
    let size = ctx.window().inner_size();

    renderer.set_viewport(size.width, size.height);

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            // Request a screen redraw.
            ctx.window().request_redraw();
        }

        Event::RedrawRequested(_) => {
            // Execute one frame's worth of instructions.
            argentum.execute_frame();

            // Render the framebuffer to the backbuffer.
            renderer.render_buffer(argentum.get_framebuffer());

            // Swap the buffers to present the scene.
            ctx.swap_buffers().unwrap();
        }

        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }

        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => handle_keyboard_input(&mut argentum, &input),

        _ => {}
    });
}
