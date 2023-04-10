#!/bin/sh

# tmux has-session -t tmux-output

# if [ $? != 0 ] 
#  then
#     tmux respawn-window -c ~/Projects/blogger/build -t 'tmux-output:build' -k '
#         cd src && make all && clear && ./blogger 2>&1 #| ~/Projects/blogger/scripts/log-parser | less'
#  else
    tmux new-window -t 'code' -a -n 'build' '
        trap "tmux kill-window" SIGINT SIGTERM
        cargo run | less -R && tmux kill-window
    '
# fi
