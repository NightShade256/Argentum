use std::os::raw::c_void;

use glow::*;

const VERT_SHADER: &str = include_str!("shaders/vert.glsl");
const FRAG_SHADER: &str = include_str!("shaders/frag.glsl");

const VERTICES: [f32; 20] = [
    1.0, 1.0, 0.0, 1.0, 0.0, 
    1.0, -1.0, 0.0, 1.0, 1.0, 
    -1.0, -1.0, 0.0, 0.0, 1.0,
    -1.0, 1.0, 0.0, 0.0, 0.0,
];

const INDICES: [u32; 6] = [0, 1, 3, 1, 2, 3];

/// Framebuffer renderer which uses OpenGL.
pub struct Renderer {
    /// The main texture that is blitted to the screen
    /// each frame.
    texture: Texture,

    /// The OpenGL context used for rendering.
    context: Context,

    /// The shader program in use.
    program: Program,

    // VAO, VBO, EBO.
    vao: VertexArray,
    vbo: Buffer,
    ebo: Buffer,
}

impl Renderer {
    /// Create a new renderer by providing a OpenGL function loader.
    pub fn new<F>(loader_function: F) -> Self
    where
        F: FnMut(&str) -> *const c_void,
    {
        unsafe {
            // Create a OpenGL context via Glow.
            let context = Context::from_loader_function(loader_function);

            // Generate VBO, VAO, EBO.
            let vao = context.create_vertex_array().unwrap();
            let vbo = context.create_buffer().unwrap();
            let ebo = context.create_buffer().unwrap();

            context.bind_vertex_array(Some(vao));

            context.bind_buffer(ARRAY_BUFFER, Some(vbo));
            context.buffer_data_u8_slice(
                ARRAY_BUFFER,
                bytemuck::cast_slice(&VERTICES),
                STATIC_DRAW,
            );

            context.bind_buffer(ELEMENT_ARRAY_BUFFER, Some(ebo));
            context.buffer_data_u8_slice(
                ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&INDICES),
                STATIC_DRAW,
            );

            context.vertex_attrib_pointer_f32(
                0,
                3,
                FLOAT,
                false,
                5 * std::mem::size_of::<f32>() as i32,
                0,
            );
            context.enable_vertex_attrib_array(0);

            context.vertex_attrib_pointer_f32(
                1,
                2,
                FLOAT,
                false,
                5 * std::mem::size_of::<f32>() as i32,
                3 * std::mem::size_of::<f32>() as i32,
            );
            context.enable_vertex_attrib_array(1);

            // Compile vertex and fragment shaders.
            let vert_shader = context
                .create_shader(VERTEX_SHADER)
                .expect("Failed to create the vertex shader.");
            let frag_shader = context
                .create_shader(FRAGMENT_SHADER)
                .expect("Failed to create the fragment shader.");

            context.shader_source(vert_shader, VERT_SHADER);
            context.compile_shader(vert_shader);

            context.shader_source(frag_shader, FRAG_SHADER);
            context.compile_shader(frag_shader);

            // Compile the shader program.
            let program = context
                .create_program()
                .expect("Failed to create shader program.");

            context.attach_shader(program, vert_shader);
            context.attach_shader(program, frag_shader);

            context.link_program(program);

            context.use_program(Some(program));

            // Delete the linked shaders.
            context.delete_shader(vert_shader);
            context.delete_shader(frag_shader);

            // Create a new empty texture.
            let texture = context
                .create_texture()
                .expect("Failed to create OpenGL texture.");

            // Bind the texture, so that we can configure it.
            context.bind_texture(TEXTURE_2D, Some(texture));

            // Set the texture filtering options
            context.tex_parameter_i32(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST as i32);
            context.tex_parameter_i32(TEXTURE_2D, TEXTURE_MIN_FILTER, NEAREST as i32);

            // Set the texture to be empty.
            context.tex_image_2d(
                TEXTURE_2D,
                0,
                RGBA as i32,
                160,
                144,
                0,
                RGBA,
                UNSIGNED_BYTE,
                None,
            );

            Self {
                texture,
                context,
                program,
                vao,
                vbo,
                ebo,
            }
        }
    }

    /// Update the internal texture with the buffer, and blit the texture to the screen.
    pub fn render_buffer(&mut self, buffer: &[u8]) {
        unsafe {
            // Recreate the texture with the buffer.
            self.context.tex_sub_image_2d(
                TEXTURE_2D,
                0,
                0,
                0,
                160,
                144,
                RGBA,
                UNSIGNED_BYTE,
                PixelUnpackData::Slice(buffer),
            );

            //self.context.bind_texture(TEXTURE_2D, Some(self.texture));
            //self.context.bind_vertex_array(Some(self.vao));
            self.context.draw_elements(TRIANGLES, 6, UNSIGNED_INT, 0);
        }
    }

    pub fn set_viewport(&mut self, width: u32, height: u32) {
        unsafe {
            self.context.viewport(0, 0, width as i32, height as i32);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.context.delete_texture(self.texture);
            self.context.delete_program(self.program);

            self.context.delete_vertex_array(self.vao);
            self.context.delete_buffer(self.vbo);
            self.context.delete_buffer(self.ebo);
        }
    }
}
