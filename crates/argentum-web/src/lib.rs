use argentum::{Argentum, ArgentumKey};
use js_sys::{Float32Array, Function, Uint8ClampedArray};
use rodio::{buffer::SamplesBuffer, OutputStream, OutputStreamHandle, Sink};
use wasm_bindgen::prelude::*;

/// Map KeyboardEvent.code to ArgentumKey.
fn map_code_to_key(code: &str) -> Option<ArgentumKey> {
    match code {
        "KeyW" => Some(ArgentumKey::Up),
        "KeyA" => Some(ArgentumKey::Left),
        "KeyS" => Some(ArgentumKey::Down),
        "KeyD" => Some(ArgentumKey::Right),
        "KeyZ" => Some(ArgentumKey::ButtonA),
        "KeyX" => Some(ArgentumKey::ButtonB),
        "Space" => Some(ArgentumKey::Select),
        "Enter" => Some(ArgentumKey::Start),

        _ => None,
    }
}

/// Handle to an instance of Argentum.
#[wasm_bindgen]
pub struct ArgentumHandle(Argentum);

#[wasm_bindgen]
impl ArgentumHandle {
    /// Create a new `ArgentumHandle` instance.
    pub fn new(rom: &[u8], callback: Function) -> Self {
        let callback = Box::new(move |buffer: &[f32]| {
            callback
                .call1(&JsValue::null(), &Float32Array::from(buffer))
                .unwrap();
        });

        Self(Argentum::new(rom, callback, None))
    }

    /// Execute a frame's worth of instructions.
    pub fn execute_frame(&mut self) {
        self.0.execute_frame();
    }

    /// Get access to the PPU framebuffer.
    pub fn get_framebuffer(&self) -> Uint8ClampedArray {
        Uint8ClampedArray::from(self.0.get_framebuffer())
    }

    /// Register a key being pressed down.
    pub fn key_down(&mut self, code: &str) {
        if let Some(key) = map_code_to_key(code) {
            self.0.key_down(key);
        }
    }

    /// Register a key being released.
    pub fn key_up(&mut self, code: &str) {
        if let Some(key) = map_code_to_key(code) {
            self.0.key_up(key);
        }
    }

    pub fn drop_handle(self) {}
}

/// Handle to a rodio Sink.
#[wasm_bindgen]
pub struct AudioHandle(OutputStream, OutputStreamHandle, Sink);

#[wasm_bindgen]
impl AudioHandle {
    /// Create a new `AudioHandle` instance.
    pub fn new() -> Self {
        let (stream, handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&handle).unwrap();

        sink.play();

        Self(stream, handle, sink)
    }

    /// Append a sound buffer to the sink.
    pub fn append(&self, buffer: &[f32]) {
        self.2.append(SamplesBuffer::new(2, 48000, buffer));
    }

    /// Get the current length of the sink.
    pub fn length(&self) -> usize {
        self.2.len()
    }

    pub fn drop_handle(self) {}
}
