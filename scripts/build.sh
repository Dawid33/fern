#!/bin/sh

cargo build --color=always 2>&1 | less -R +F

