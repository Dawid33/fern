sudo perf script record -F997 --call-graph dwarf,16384 -e cpu-clock ./target/x86_64-unknown-linux-gnu/debug/fern
sudo chmod +r perf.data
hotspot