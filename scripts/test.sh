#!/bin/sh

tmux has-session -t tmux-output

if [ $? = 0 ] 
 then
    tmux respawn-window -c ~/Projects/blogger/build -t 'tmux-output:build' -k '
        make && clear && make test'
 else
    tmux new-window -c ~/Projects/blogger/build -t 'code' -a -n 'test' '
        make && clear && ctest --output-on-failure 2>&1 | less -R
    '
fi
