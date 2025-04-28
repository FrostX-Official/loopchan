FROM rust:1.86.0-alpine

RUN apk add musl-dev

WORKDIR /app

COPY ./Cargo.toml ./

RUN mkdir src && echo "fn main() {}" > src/main.rs

COPY ./src ./src
RUN cargo build --release

CMD ./target/release/loopchan