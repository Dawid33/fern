#!/bin/sh

tmux has-session -t tmux-output

if [ $? = 0 ] 
 then
    # Dual monitor mode with extra tmux session to show output.
    # tmux respawn-window -c ~/Projects/fern -t 'tmux-output:build' -k '
    #     cd build && clear && make all 2>&1 | less -R'
    echo "Wrong"
 else
    # Single monitor mode.
    tmux new-window -c ~/Projects/fern -t 'code' -a -n 'build' '
        trap "tmux kill-window" SIGINT SIGTERM
        cargo build 2>&1 | less -R +F && tmux kill-window'
fi

