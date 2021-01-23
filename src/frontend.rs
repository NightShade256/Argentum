use glium::glutin::ContextBuilder;
use glium::glutin::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use glium::{texture::RawImage2d, uniforms::MagnifySamplerFilter};
use glium::{BlitTarget, Display, Surface, Texture2d};

use argentum_core::gameboy::GameBoy;

/// Initialize the window, and then glium's
/// display.
fn initialize_display(event_loop: &EventLoop<()>) -> Display {
    // Create window and OpenGL context builders.
    let cb = ContextBuilder::new().with_vsync(true);

    let wb = WindowBuilder::new()
        .with_decorations(true)
        .with_title("Argentum GB")
        .with_min_inner_size(LogicalSize::new(160, 144))
        .with_inner_size(LogicalSize::new(480, 432));

    // Create a Glium display.
    let display = Display::new(wb, cb, event_loop).expect("Failed to create display.");

    // Clear the display.
    let mut frame = display.draw();
    frame.clear_color(0.0, 0.0, 0.0, 1.0);
    frame.finish().expect("Failed to swap buffers.");

    display
}

/// Start running the emulator.
pub fn start() {
    // Create GB instance and load a ROM.
    let rom_path = std::env::args()
        .nth(1)
        .expect("Please provide a ROM to execute.");
    let rom = std::fs::read(rom_path).expect("Failed to read the ROM.");

    let mut argentum = GameBoy::new(&rom);

    // Create a event loop, and initialize the display.
    let event_loop = EventLoop::new();
    let display = initialize_display(&event_loop);

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            display.gl_window().window().request_redraw();
        }

        Event::RedrawRequested(_) => {
            // Execute one frame's worth of instructions.
            argentum.execute_frame();

            let framebuffer = argentum.get_framebuffer();

            let mut frame = display.draw();
            frame.clear_color(0.0, 0.0, 0.0, 1.0);

            // Create a texture out of the framebuffer.
            let image = RawImage2d::from_raw_rgba_reversed(framebuffer, (160, 144));
            let texture =
                Texture2d::new(&display, image).expect("Failed to create OpenGL texture.");

            // Blit the texture onto the screen.
            let window_size = display.gl_window().window().inner_size();

            texture.as_surface().blit_whole_color_to(
                &frame,
                &BlitTarget {
                    left: 0,
                    bottom: 0,
                    width: window_size.width as i32,
                    height: window_size.height as i32,
                },
                MagnifySamplerFilter::Nearest,
            );

            frame.finish().expect("Failed to swap buffers.");
        }

        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }

        _ => {}
    });
}
