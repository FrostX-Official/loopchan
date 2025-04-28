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
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
RUN cargo build --release

CMD ./target/release/loopchan