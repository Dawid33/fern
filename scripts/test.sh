#!/bin/sh

cargo test tree_ --features="build-binary" --target=x86_64-unknown-linux-gnu --color=always 2>&1 | less -R +F
