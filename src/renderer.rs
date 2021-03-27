use std::os::raw::{c_uint, c_void};

// OpenGL 3.3 core bindings by Lokathor.
use gl33::*;

/// The default shaders for the renderer written in GLSL.
const VERT_SHADER_SOURCE: &str = include_str!("shaders/vert.glsl");
const FRAG_SHADER_SOURCE: &str = include_str!("shaders/frag.glsl");

/// Two triangles with their vertex positions
/// and textures coordinates.
const VERTICES: [f32; 30] = [
    // First Triangle
    -1.0, 1.0, 0.0, 0.0, 0.0, -1.0, -1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0,
    // Second Triangle
    1.0, -1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, -1.0, -1.0, 0.0, 0.0, 1.0,
];

/// Framebuffer renderer which uses OpenGL.
pub struct Renderer {
    /// The OpenGL context used for rendering.
    context: GlFns,

    /// The OpenGL texture that is drawn every frame.
    texture: c_uint,

    /// Vertex Array Object.
    vao: c_uint,

    /// Vertex Buffer Object.
    vbo: c_uint,

    /// The shader program in use.
    program: c_uint,
}

impl Renderer {
    /// Create a new renderer by providing a OpenGL function loader.
    pub fn new<F>(loader_function: F) -> Self
    where
        F: Fn(*const u8) -> *const c_void,
    {
        unsafe {
            // Create a OpenGL context via Glow.
            let context = GlFns::load_from(&loader_function)
                .expect("Failed to load OpenGL 3.3 core functions.");

            // Generate VAO.
            let mut vao: c_uint = 0;

            context.GenVertexArrays(1, &mut vao as _);

            // Generate VBO.
            let mut vbo: c_uint = 0;

            context.GenBuffers(1, &mut vbo as _);

            // Bind VBO and fill it with vertex data.
            context.BindVertexArray(vao);

            context.BindBuffer(GL_ARRAY_BUFFER, vbo);
            context.BufferData(
                GL_ARRAY_BUFFER,
                std::mem::size_of_val(&VERTICES) as isize,
                VERTICES.as_ptr() as _,
                GL_STATIC_DRAW,
            );

            // Setup vertex attribute pointers.
            context.VertexAttribPointer(
                0,
                3,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                5 * std::mem::size_of::<f32>() as i32,
                std::ptr::null(),
            );

            context.EnableVertexAttribArray(0);

            context.VertexAttribPointer(
                1,
                2,
                GL_FLOAT,
                GL_FALSE.0 as u8,
                5 * std::mem::size_of::<f32>() as i32,
                (3 * std::mem::size_of::<f32>()) as *const c_void,
            );

            context.EnableVertexAttribArray(1);

            // Compile vertex and fragment shaders.
            let vert_shader = context.CreateShader(GL_VERTEX_SHADER);
            let frag_shader = context.CreateShader(GL_FRAGMENT_SHADER);

            context.ShaderSource(
                vert_shader,
                1,
                &VERT_SHADER_SOURCE.as_ptr() as _,
                &(VERT_SHADER_SOURCE.len() as i32) as _,
            );
            context.CompileShader(vert_shader);

            context.ShaderSource(
                frag_shader,
                1,
                &FRAG_SHADER_SOURCE.as_ptr() as _,
                &(FRAG_SHADER_SOURCE.len() as i32) as _,
            );
            context.CompileShader(frag_shader);

            // Compile the shader program.
            let program = context.CreateProgram();

            context.AttachShader(program, vert_shader);
            context.AttachShader(program, frag_shader);

            context.LinkProgram(program);
            context.UseProgram(program);

            // Delete the linked shaders.
            context.DeleteShader(vert_shader);
            context.DeleteShader(frag_shader);

            // Create a new empty texture.
            let mut texture: c_uint = 0;

            context.GenTextures(1, &mut texture as _);

            // Bind the texture, so that we can configure it.
            context.BindTexture(GL_TEXTURE_2D, texture);

            // Set the texture filtering options
            context.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST.0 as i32);
            context.TexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST.0 as i32);

            // Set the texture to be empty.
            context.TexImage2D(
                GL_TEXTURE_2D,
                0,
                GL_RGBA.0 as i32,
                160,
                144,
                0,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                std::ptr::null(),
            );

            Self {
                context,
                texture,
                vao,
                vbo,
                program,
            }
        }
    }

    /// Update the internal texture with the buffer, and blit the texture to the screen.
    pub fn render_buffer(&mut self, buffer: &[u8]) {
        unsafe {
            // Recreate the texture with the buffer.
            self.context.TexSubImage2D(
                GL_TEXTURE_2D,
                0,
                0,
                0,
                160,
                144,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                buffer.as_ptr() as _,
            );

            self.context.DrawArrays(GL_TRIANGLES, 0, 6);
        }
    }

    pub fn set_viewport(&mut self, width: i32, height: i32) {
        unsafe {
            self.context.Viewport(0, 0, width, height);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.context.DeleteTextures(1, &self.texture as _);
            self.context.DeleteVertexArrays(1, &self.vao as _);
            self.context.DeleteBuffers(1, &self.vbo as _);
            self.context.DeleteProgram(self.program);
        }
    }
}
