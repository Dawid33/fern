
#!/bin/sh

cargo build --target=wasm32-unknown-unknown --color=always --release 2>&1 | less -R +F

# wasm-pack build --out-dir web/fern --target web --out-name fern