use fermium::prelude::*;

pub struct Renderer {
    /// SDL Renderer used to blit the texture to the screen.
    renderer: *mut SDL_Renderer,

    /// Texture updated every frame.
    texture: *mut SDL_Texture,
}

impl Renderer {
    /// Create a new `Renderer` instance.
    pub fn new(window: *mut SDL_Window) -> Self {
        unsafe {
            let renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED.0);

            let texture = SDL_CreateTexture(
                renderer,
                SDL_PIXELFORMAT_RGBA32.0,
                SDL_TEXTUREACCESS_STREAMING.0,
                160,
                144,
            );

            Self { renderer, texture }
        }
    }

    /// Update the texture and present the changes.
    pub fn update_texture(&mut self, buffer: &[u8]) {
        unsafe {
            SDL_UpdateTexture(
                self.texture,
                std::ptr::null(),
                buffer.as_ptr() as _,
                4 * 160,
            );

            SDL_RenderCopy(
                self.renderer,
                self.texture,
                std::ptr::null(),
                std::ptr::null(),
            );

            SDL_RenderPresent(self.renderer);
        }
    }
}
