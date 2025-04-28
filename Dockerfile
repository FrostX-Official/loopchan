FROM rust:1.86.0-alpine

RUN apk add musl-dev

WORKDIR /app

COPY ./ ./

COPY ./src ./src
RUN cargo build --release

CMD ./target/release/loopchan