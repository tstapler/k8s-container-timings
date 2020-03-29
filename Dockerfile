FROM rust:1.41 as build
WORKDIR /opt/blank
# Create blank project
RUN USER=root cargo new PROJ
COPY Cargo.toml /opt/blank/PROJ
COPY Cargo.lock /opt/blank/PROJ
WORKDIR /opt/blank/PROJ
WORKDIR /opt/app
COPY . .
ENV CARGO_HTTP_DEBUG=true
ENV CARGO_LOG=cargo::ops::registry=debug
RUN cargo build --verbose
RUN cargo build --release
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y extra-runtime-dependencies
COPY --from=build --chown=nonroot:nonroot /opt/app/target/release/controller /app/
EXPOSE 8080
ENTRYPOINT ["/app/controller"]
