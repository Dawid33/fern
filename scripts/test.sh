#!/bin/sh

tmux has-session -t tmux-output

if [ $? = 0 ] 
 then
    # Dual monitor mode with extra tmux session to show output.
    tmux respawn-window -c ~/Projects/pm -t 'tmux-output:build' -k '
        cmake -B build && cmake --build build --target lua-external
        cd build && clear && make test 2>&1 | less -R'
 else
    # Single monitor mode.
    tmux new-window -c ~/Projects/pm -t 'code' -a -n 'build' '
        trap "tmux kill-window" SIGINT SIGTERM
        cmake -B build && cmake --build build --target lua-external && clear
        cmake -B build --target test 2>&1 | less -R && tmux kill-window'
fi

