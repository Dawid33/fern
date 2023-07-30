#!/bin/bash

SESSIONNAME="fern"
tmux has-session -t $SESSIONNAME &> /dev/null


if [ $? != 0 ] 
 then
    cd ~/Projects/fern
    tmux new-session -s $SESSIONNAME -n 'code' -d bash -c "hx" \;\
      new-window -t $SESSIONNAME -n 'terminal' -d 'bash' \;\
      new-window -t $SESSIONNAME -n 'ranger' -d 'ranger'
fi

tmux attach
