use argentum::Argentum;
use rodio::OutputStreamHandle;
use rodio::{buffer::SamplesBuffer, queue::queue, OutputStream};
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, ImageData};

#[wasm_bindgen]
extern "C" {
    fn setInterval(closure: &Closure<dyn FnMut()>, time: u32) -> i32;
    fn clearInterval(interval_id: i32);
}

#[wasm_bindgen(module = "/src/js/snippet.js")]
extern "C" {
    fn paint_ctx(img_data: &ImageData, ctx: &CanvasRenderingContext2d);
}

#[wasm_bindgen]
pub struct EmulatorHandle {
    pub interval_id: i32,

    _stream: OutputStream,
    _output_handle: OutputStreamHandle,
    _closure: Closure<dyn FnMut()>,
}

#[wasm_bindgen]
pub fn start_emulator(rom: Vec<u8>, ctx: CanvasRenderingContext2d) -> EmulatorHandle {
    let (stream, output_handle) =
        OutputStream::try_default().expect("could not initialize output stream");

    let (input, output) = queue::<f32>(true);

    output_handle
        .play_raw(output)
        .expect("could not playback audio queue");

    let mut argentum = Argentum::new(
        &rom,
        Box::new(move |buffer| {
            input.append(SamplesBuffer::new(2, 48000, buffer));
        }),
        None,
    );

    let closure = Closure::wrap(Box::new(move || {
        argentum.execute_frame();

        let framebuffer = argentum.get_framebuffer();

        let image_data =
            web_sys::ImageData::new_with_u8_clamped_array(Clamped(&framebuffer), 160).unwrap();

        paint_ctx(&image_data, &ctx);
    }) as Box<dyn FnMut()>);

    let interval_id = setInterval(&closure, 16);

    EmulatorHandle {
        interval_id,
        _stream: stream,
        _output_handle: output_handle,
        _closure: closure,
    }
}

#[wasm_bindgen]
pub fn stop_emulator(handle: Option<EmulatorHandle>) {
    if let Some(handle) = handle {
        clearInterval(handle.interval_id);
    }
}
