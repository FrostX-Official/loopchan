FROM rust:1.86.0-alpine
RUN apk add musl-dev
WORKDIR /app

# Build deps before building loopchan, caching compiled deps without need to rebuild them
COPY ./Cargo.toml ./
COPY ./Cargo.lock ./
COPY ./loopchan.ico ./
COPY ./build.rs ./
RUN mkdir ./src && echo 'fn main() { println!("Dummy!"); }' > ./src/main.rs
RUN cargo build --release
RUN rm -rf ./src
COPY ./src ./src
RUN touch -a -m ./src/main.rs
RUN cargo build --release

LABEL org.opencontainers.image.authors="frostx-official"
LABEL org.opencontainers.image.source="https://github.com/frostx-official/loopchan"
LABEL org.opencontainers.image.description="A discord bot written in Rust for PTL discord community."

CMD ./target/release/loopchan