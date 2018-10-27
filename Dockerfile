FROM clux/muslrust:nightly

WORKDIR /build

# Cache downloaded packages to avoid redownloading
# all dependencies every time a project file is 
# changed on the server. Since this project downloads
# a large number of dependencies, this should save 
# a decent amount of bandwith
ADD Cargo.toml Cargo.lock /build/
RUN mkdir src

# Fetch all dependencies to save bandwith
RUN echo "fn main() {}" > src/main.rs
RUN cargo fetch
RUN rm -rf src

ADD . /build

RUN cargo build --release
RUN mkdir /artifacts
RUN mv target/x86_64-unknown-linux-musl/release/airmash-front-page-server /artifacts/server

FROM alpine:latest

COPY --from=0 /artifacts/server /server

ENV RUST_LOG=info

ENTRYPOINT [ "/server" ]
