FROM rust:1.43 as build

RUN apt-get update
RUN apt-get install python
RUN apt-get install -y sqlite3 libsqlite3-dev

WORKDIR /app

RUN USER=root cargo new --bin crate_analyzer
WORKDIR /app/crate_analyzer
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN cargo build --release
RUN rm src/*.rs
RUN rm -f target/release/deps/crate_analyzer*

COPY src src
COPY grapher.py grapher.py
RUN cargo build --release
RUN cargo install --path .

ENV RUST_BACKTRACE=1
RUN mkdir /data
RUN mkdir /data/praezi
RUN mkdir /data/praezi/batch
RUN mkdir /data/praezi/batch/data/

CMD ["./target/release/crate_analyzer"]