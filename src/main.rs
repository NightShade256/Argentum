use std::{env, ffi::CString, path::PathBuf};

use argentum_core::{GameBoy, GbKey};
use clap::Clap;
use fermium::prelude::*;

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
        let rom = std::fs::read(opts.rom_file).expect("Failed to read the ROM file.");

        // Create a Game Boy instance and skip the bootrom.
        let mut argentum = GameBoy::new(
            &rom,
            Box::new(|buffer| {
                while SDL_GetQueuedAudioSize(SDL_AudioDeviceID(1)) > 1024 * 4 {
                    SDL_Delay(1);
                }

                SDL_QueueAudio(
                    SDL_AudioDeviceID(1),
                    buffer.as_ptr() as _,
                    (std::mem::size_of::<f32>() * buffer.len()) as u32,
                );
            }),
        );

        argentum.skip_bootrom();

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

        audio_spec.freq = 48000;
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

        // De-init SDL subsystems, and return.
        SDL_CloseAudio();
        SDL_GL_DeleteContext(context);
        SDL_DestroyWindow(window);
        SDL_Quit();
    }
}
