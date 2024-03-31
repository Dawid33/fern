#!/bin/sh

cargo run --color=always --features="build-binary" 2>&1 | less -R +F 
# cargo run --color=always --target=x86_64-unknown-linux-gnu --features="build-binary" 2>&1 | less -R +F 
