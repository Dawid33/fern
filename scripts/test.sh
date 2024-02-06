#!/bin/sh

cargo test tree_ --color=always 2>&1 | less -R +F
