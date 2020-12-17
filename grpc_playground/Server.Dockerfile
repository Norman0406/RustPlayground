FROM rust:latest as build

RUN rustup component add rustfmt --toolchain 1.48.0-x86_64-unknown-linux-gnu

WORKDIR /app
COPY ./proto/ ./proto
COPY ./server/ ./server/
RUN cd server && cargo build --release

FROM debian:buster-slim
COPY --from=build /app/server/target/release/server /app/

EXPOSE 50001
CMD ["/app/server", "-p", "50001"]
