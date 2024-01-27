#!/bin/bash

cat nfa.dot | 
  dot -Gmargin=0.7 '-Gbgcolor=#ffffff00' -Gfontname=CascadiaCode -Gcolor=#d5c4a1 -Gfontcolor=#d5c4a1 -Ecolor=#d5c4a1 -Efontcolor=#d5c4a1  -Ncolor=#d5c4a1 -Nfontcolor=#d5c4a1 -Ecolor=white -T png | 
  kitty +kitten icat && read -n 1 -s -r -p "Press any key to continue"

