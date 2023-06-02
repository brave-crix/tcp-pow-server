FROM rust:1.69-buster as builder

COPY . .
RUN cargo test
RUN cargo build --release

FROM debian:bookworm

COPY --from=builder ./target/release/server /usr/local/bin
COPY --from=builder ./target/release/client /usr/local/bin

ENV PORT 8080
EXPOSE 8080

CMD ["server"]
