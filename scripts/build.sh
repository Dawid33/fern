#!/bin/sh

# tmux has-session -t "tmux-output" &> /dev/null

# if [ $? != 0 ] 
#  then
    # Dual monitor mode with extra tmux session to show output.
    # tmux respawn-window  -t 'tmux-output:build' -k '
    #     cargo build'
# else
    # Single monitor mode.
    tmux new-window -t 'code' -a -n 'build' '
        cargo build --color always 2>&1 | less -R'
# fi

