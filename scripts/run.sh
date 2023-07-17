#!/bin/sh

tmux has-session -t tmux-output

if [ $? = 0 ] 
 then
    # Dual monitor mode
    tmux respawn-window -c ~/Projects/fern -t 'tmux-output:build' -k '
        echo Hello | less && tmux kill-window'
 else
    # Single monitor mode
    tmux new-window -c ~/Projects/fern -t 'code' -a -n 'build' '
        trap "tmux kill-window" SIGINT SIGTERM
        cargo run 2>/dev/null | less -R && tmux kill-window
    '
fi
