export function paint_ctx(img_data, ctx) {
    createImageBitmap(img_data, {
        "resizeWidth": 480,
        "resizeHeight": 432,
        "resizeQuality": "pixelated"
    }).then((bitmap) => {
        ctx.drawImage(bitmap, 0.0, 0.0);
    });
}
