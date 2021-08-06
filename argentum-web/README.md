# How to Compile and Run

Note: You need the `wasm-pack` utility to be installed.

To compile, 

```bash
cd argentum-web
...

wasm-pack build --target web --out-dir www/wasm
...
```

then to run,

```bash
# Spawn a HTTP server in www/ folder
basic-http-server ./www
```

and navigate to localhost in your web browser.
