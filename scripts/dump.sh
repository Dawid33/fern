#!/bin/bash

cat ptree.dot | dot -Gmargin=0.7 -Gfontname=CascadiaCode -T png > ptree.png
cat ast.dot | dot -Gmargin=0.7 -Gfontname=CascadiaCode -T png > ast.png
cat dfa.dot | dot -Gmargin=0.7 -Gfontname=CascadiaCode -T png > dfa.png
cat nfa.dot | dot -Gmargin=0.7 -Gfontname=CascadiaCode -T png > nfa.png
cat keyword_dfa.dot | dot -Gmargin=0.7 -Gfontname=CascadiaCode -T png > keyword_dfa.png
cat keyword_nfa.dot | dot -Gmargin=0.7 -Gfontname=CascadiaCode -T png > keyword_nfa.png
cat nfa.dot | dot -Gmargin=0.7 -Gfontname=CascadiaCode -T png > nfa.png

