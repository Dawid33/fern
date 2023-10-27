FROM ubuntu as first

RUN apt-get update

RUN apt-get install -y \
    build-essential \
    curl \
    less

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain nightly-2023-10-10 -y

RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
ENV PATH="${PATH}:/root/.cargo/bin"
RUN rustup default nightly-2023-10-10
RUN rustup target add wasm32-unknown-unknown --toolchain nightly-2023-10-10
RUN rustup component add rust-src --toolchain nightly-2023-10-10-x86_64-unknown-linux-gnu
RUN rustup component add rust-std --toolchain nightly-2023-10-10-x86_64-unknown-linux-gnu
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | bash

ADD . /app
WORKDIR /app
RUN cargo build --target=wasm32-unknown-unknown --color=always --release
RUN wasm-pack build --out-dir web/public --target web --out-name fern --no-pack

FROM oven/bun as second

COPY --from=first /app/web /app

WORKDIR /app
ADD web/package.json /app/package.json
RUN bun install
run cat /app/package.json
RUN bun build index.js --outdir=public

FROM jekyll/jekyll as third

COPY --from=second /app /app
WORKDIR /app

RUN bundle 
RUN chown -R 1000:1000 * 
RUN jekyll build --trace 

FROM nginx
COPY --from=third /app/public /usr/share/nginx/html
