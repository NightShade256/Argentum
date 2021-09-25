import init, { ArgentumHandle, AudioHandle } from "../wasm/argentum_web.js"

const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");

const rom_input = document.getElementById("rom_input");
const start = document.getElementById("start");
const stop = document.getElementById("stop");

ctx.fillStyle = "black";
ctx.fillRect(0.0, 0.0, canvas.width, canvas.height);

async function main() {
    await init();

    let rom_file = null;
    let argentum = null;
    let running = false;

    rom_input.oninput = (_) => {
        rom_file = rom_input.files[0];
    };

    start.onclick = (_) => {
        if (rom_file === null || argentum !== null) {
            return;
        }

        let reader = new FileReader();

        reader.readAsArrayBuffer(rom_file);
        reader.onloadend = (_) => {
            let rom = new Uint8Array(reader.result);
            running = true;

            let audio = AudioHandle.new();
            let intervalID = null;

            argentum = ArgentumHandle.new(rom, (buffer) => {
                audio.append(buffer);
            });

            function main_loop() {
                if (!running) {
                    argentum.drop_handle();
                    audio.drop_handle();

                    argentum = null;
                    audio = null;

                    clearInterval(intervalID);
                    return;
                }

                if (audio.length() < 15) {
                    /* execute frame's worth of instructions */
                    argentum.execute_frame();

                    /* paint the frame */
                    let framebuffer = argentum.get_framebuffer();
                    let image_data = new ImageData(framebuffer, 160, 144);

                    createImageBitmap(image_data, {
                        resizeQuality: "pixelated",
                        resizeWidth: 480,
                        resizeHeight: 432,
                    }).then((bitmap) => {
                        ctx.drawImage(bitmap, 0.0, 0.0);
                    });
                }
            }

            intervalID = setInterval(main_loop, 1);
        };
    };

    stop.onclick = (_) => {
        running = false;
        ctx.fillRect(0.0, 0.0, canvas.width, canvas.height);
    };

    document.onkeydown = (event) => {
        if (argentum !== null) {
            argentum.key_down(event.code);
        }
    }

    document.onkeyup = (event) => {
        if (argentum !== null) {
            argentum.key_up(event.code);
        }
    }
}

main()
