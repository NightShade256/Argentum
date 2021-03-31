use std::{env, ffi::CString, path::PathBuf};

use argentum_core::{GameBoy, GbKey};
use clap::Clap;
use fermium::prelude::*;

mod renderer;

use renderer::Renderer;

/// The version of this crate. To pass to Clap CLI.
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clap)]
#[clap(name = "Argentum GB")]
#[clap(version = PKG_VERSION, about = "A Game Boy emulator written in Rust.")]
struct Opt {
    /// The Game Boy ROM file to execute.
    #[clap(parse(from_os_str))]
    rom_file: PathBuf,

    /// Turn on basic logging support.
    #[clap(short, long)]
    logging: bool,

    /// Skip the bootrom (Optix's custom bootrom Bootix).
    #[clap(short, long)]
    skip_bootrom: bool,
}

/// Handle keyboard input.
fn handle_keyboard_input(gb: &mut GameBoy, input: SDL_Scancode, is_pressed: bool) {
    let key = match input {
        SDL_SCANCODE_W => Some(GbKey::UP),
        SDL_SCANCODE_A => Some(GbKey::LEFT),
        SDL_SCANCODE_S => Some(GbKey::DOWN),
        SDL_SCANCODE_D => Some(GbKey::RIGHT),
        SDL_SCANCODE_RETURN => Some(GbKey::START),
        SDL_SCANCODE_SPACE => Some(GbKey::SELECT),
        SDL_SCANCODE_Z => Some(GbKey::BUTTON_A),
        SDL_SCANCODE_X => Some(GbKey::BUTTON_B),

        _ => None,
    };

    if let Some(key) = key {
        if is_pressed {
            gb.key_down(key);
        } else {
            gb.key_up(key);
        }
    }
}

/// Start running the emulator.
pub fn main() {
    unsafe {
        // Parse command line arguments.
        let opts: Opt = Opt::parse();

        // Setup logging.
        if opts.logging {
            env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
        }

        // Read the ROM file into memory.
        let mut rom_path = opts.rom_file;

        let rom = std::fs::read(&rom_path).expect("Failed to read the ROM file.");

        // Check if there is a save file.
        rom_path.set_extension("sav");

        let save_file = std::fs::read(&rom_path).ok();

        // Create a Game Boy instance and skip the bootrom.
        let mut argentum = GameBoy::new(
            &rom,
            Box::new(|buffer| {
                while SDL_GetQueuedAudioSize(SDL_AudioDeviceID(1)) > 1024 * 4 * 2 {
                    SDL_Delay(1);
                }

                SDL_QueueAudio(
                    SDL_AudioDeviceID(1),
                    buffer.as_ptr() as _,
                    (std::mem::size_of::<f32>() * buffer.len()) as u32,
                );
            }),
            save_file,
        );

        if opts.skip_bootrom {
            argentum.skip_bootrom();
        }

        // Initialize SDL's video and audio subsystems.
        if SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO | SDL_INIT_TIMER) != 0 {
            panic!("Failed to initialize SDL.");
        }

        // Set OpenGL attributes.
        SDL_GL_SetAttribute(
            SDL_GL_CONTEXT_PROFILE_MASK,
            SDL_GL_CONTEXT_PROFILE_CORE.0 as i32,
        );

        SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3);
        SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 3);

        // Create a SDL window, and an OpenGL context.
        let title = CString::new("Argentum GB").unwrap();

        let window = SDL_CreateWindow(
            title.as_ptr(),
            SDL_WINDOWPOS_CENTERED,
            SDL_WINDOWPOS_CENTERED,
            480,
            432,
            SDL_WINDOW_OPENGL.0,
        );

        let context = SDL_GL_CreateContext(window);

        // Set the window icon.
        let mut logo_bytes = include_bytes!("images/argentum_logo.rgb").to_vec();

        let icon_surface = SDL_CreateRGBSurfaceWithFormatFrom(
            logo_bytes.as_mut_ptr() as _,
            128,
            128,
            24,
            3 * 128,
            SDL_PIXELFORMAT_RGB24.0,
        );

        SDL_SetWindowIcon(window, icon_surface);
        SDL_FreeSurface(icon_surface);

        // Make the context, "current".
        SDL_GL_MakeCurrent(window, context);

        // Create our renderer instance, and set OpenGL viewport.
        let mut renderer = Renderer::new(|s| SDL_GL_GetProcAddress(s as _));

        let mut w: i32 = 0;
        let mut h: i32 = 0;

        SDL_GL_GetDrawableSize(window, &mut w as _, &mut h as _);

        renderer.set_viewport(w, h);

        // Setup SDL audio system.
        let mut audio_spec: SDL_AudioSpec = std::mem::zeroed();

        audio_spec.freq = 65536;
        audio_spec.format = AUDIO_F32SYS;
        audio_spec.channels = 2;
        audio_spec.samples = 1024;
        audio_spec.callback = None;

        // Open audio queue with the desired spec.
        SDL_OpenAudio(&mut audio_spec as _, std::ptr::null_mut());

        // Start the audio queue.
        SDL_PauseAudio(0);

        // Used to store the current polled event.
        let mut event: SDL_Event = std::mem::zeroed();

        'main: loop {
            // Poll events, quit and handle input appropriately.
            while SDL_PollEvent(&mut event as _) != 0 {
                match event.type_ {
                    SDL_KEYDOWN => {
                        handle_keyboard_input(&mut argentum, event.key.keysym.scancode, true);
                    }

                    SDL_KEYUP => {
                        handle_keyboard_input(&mut argentum, event.key.keysym.scancode, false);
                    }

                    SDL_QUIT => break 'main,

                    _ => {}
                }
            }

            // Execute one frame's worth of instructions.
            argentum.execute_frame();

            // Render the framebuffer to the backbuffer.
            renderer.render_buffer(argentum.get_framebuffer());

            // Swap front and back buffers.
            SDL_GL_SwapWindow(window);
        }

        if let Some(ram_save) = argentum.get_ram_dump() {
            std::fs::write(&rom_path, &ram_save).expect("Failed to write save file.");
        }

        // De-init SDL subsystems, and return.
        SDL_CloseAudio();
        SDL_GL_DeleteContext(context);
        SDL_DestroyWindow(window);
        SDL_Quit();
    }
}
