#!/bin/sh

tmux has-session -t tmux-output

if [ $? = 0 ] 
 then
    tmux respawn-pane -c ~/Projects/blogger -t 'tmux-output:build' -k '
        cd scripts && 
        python3 .gdbinit.py &&
        valgrind.bin --vgdb=full --vgdb-error=0 ../build/src/blogger ../build/src/dummy_data/test.toml
        ' 

    tmux new-window -t 'code' -c ~/Projects/blogger -a -n 'gdb' '
        trap "tmux kill-window" SIGINT SIGTERM
        cd scripts && gdb -ex "target remote | vgdb" -x .gdbinit --tui && tmux kill-window'

    tmux send-keys c
    tmux send-keys Enter
 else
    tmux new-window -t 'code' -c ~/Projects/blogger -a -n 'gdb' '
        trap "tmux kill-window" SIGINT SIGTERM
        tmux split-window -h "cd scripts && gdbserver localhost:1234 ..build/src/blogger ..build/src/dummy_data/test.toml 2>&1 | less -R && tmux kill-window" && \
        tmux select-pane -L && \
        cd scripts && gdb -ex "target remote localhost:1234" --tui -x .gdbinit && tmux kill-window'

    tmux send-keys y
    tmux send-keys c
    tmux send-keys Enter

    tmux send-keys c
    tmux send-keys Enter
fi
