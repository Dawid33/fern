
#!/bin/sh

# RUSTFLAGS="-C codegen-units=1 -C target-feature=+atomics,+bulk-memory,+mutable-globals -C link-arg=--export=__stack_pointer -C opt-level=z" cargo +nightly build --target=wasm32-unknown-emscripten --release -Z build-std=panic_abort,std  
# RUSTFLAGS="-C codegen-units=1 -C target-feature=+atomics,+bulk-memory,+mutable-globals -C link-arg=--no-entry -C opt-level=z" cargo +nightly build --target=wasm32-unknown-emscripten --release -Z build-std=panic_abort,std 2>&1 | less -R +F  

# wasm-bindgen --target web --out-dir web/fern ./target/wasm32-unknown-emscripten/release/ferncore.wasm 

cargo build --target=wasm32-unknown-unknown --color=always --release 2>&1 | less -R +F  
wasm-pack build --out-dir web/public --target web --out-name fern