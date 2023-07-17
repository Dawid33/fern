#!/bin/sh

tmux has-session -t tmux-output 

if [ $? = 0 ] 
 then
    tmux respawn-window -c ~/Projects/pm -t 'tmux-output:build' -k '
        cmake -B build -DCMAKE_BUILD_TYPE=Debug
        cmake --build build && clear
        gdbserver localhost:1234 ./build/src/pm'

    tmux new-window -t 'code' -c ~/Projects/pm -a -n 'gdb' '
        trap "tmux kill-window" SIGINT SIGTERM
        gdb -ex "target remote localhost:1234" -ex "source .gdbinit" --tui --quiet && tmux kill-window '

    tmux send-keys c
    tmux send-keys Enter

    tmux send-keys c
    tmux send-keys Enter
 else
    echo "Not implmented yet"
fi
