
OUTPUT_DIR=$(git rev-parse HEAD);
mkdir $OUTPUT_DIR;
cargo criterion --message-format=json | jq -c 'if .reason == "benchmark-complete" then . else empty end' | while read -r benchmark; do
  echo $benchmark > $OUTPUT_DIR/$(echo $benchmark | jq -r '.id')
done
