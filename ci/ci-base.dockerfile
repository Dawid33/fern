FROM rust:1.67
RUN CARGO_HOME=/cargo cargo install cargo-criterion
RUN apt-get update && apt-get install -y jq lftp