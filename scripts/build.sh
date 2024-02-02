#!/bin/sh

cargo build --features="build-binary" --target=x86_64-unknown-linux-gnu --color=always 2>&1 | less -R +F

# Web build
# cargo build --target=wasm32-unknown-unknown --color=always --release 2>&1 | less -R +F  
# wasm-pack build --out-dir web/public --target web --out-name fern

