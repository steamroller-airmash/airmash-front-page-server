FROM clux/muslrust:stable

WORKDIR /build

ADD . /build

RUN cargo build --release
RUN mkdir /artifacts
RUN mv target/x86_64-unknown-linux-musl/release/airmash-front-page-server /artifacts/server

FROM alpine:latest

COPY --from=0 /artifacts/server /server

ENV RUST_LOG=info

ENTRYPOINT [ "/server" ]
