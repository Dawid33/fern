#!/bin/bash

OUTPUT_DIR=$(git rev-parse HEAD);
mkdir "$OUTPUT_DIR";
echo -e "{\n\t\"files\": [" > "$OUTPUT_DIR/manifest.json"

declare -a files=()
while read -r BENCHMARK; do
  FILE=$(echo "$BENCHMARK" | jq -r '.id').json
  files+=("$FILE")
  touch "$OUTPUT_DIR"/$FILE
  echo "$BENCHMARK" > "$OUTPUT_DIR/$FILE"
done < <(cargo criterion --message-format=json | jq -c 'if .reason == "benchmark-complete" then . else empty end')

# get length of an array
filecount=${#files[@]}

# use for loop to read all values and indexes
for (( i=0; i < filecount; i++ ));
do
  if [ $((filecount - 1)) == $i ];
  then
    echo -e "\t\t\"${files[$i]}\"" >> $OUTPUT_DIR/manifest.json
  else
    echo -e "\t\t\"${files[$i]}\"," >> $OUTPUT_DIR/manifest.json
  fi
done

echo -e "\t]," >> $OUTPUT_DIR/manifest.json

echo -e "\t\"time\": \"$(git show -s --format=%cI)\"\n}" >> $OUTPUT_DIR/manifest.json

printf "%s " "${DP[@]}"