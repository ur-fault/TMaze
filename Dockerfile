FROM rust:alpine

RUN apk add --no-cache musl-dev

COPY . /usr/src/tmaze
WORKDIR /usr/src/tmaze

RUN cargo build --release --all-features

CMD ["cargo", "run", "--release"]