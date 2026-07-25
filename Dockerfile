FROM --platform=$BUILDPLATFORM rust:1.92.0-bullseye AS builder

# cross-compile for amd64 natively on the build host (avoids qemu emulation)
RUN apt-get update && apt-get install -y --no-install-recommends gcc-x86-64-linux-gnu libc6-dev-amd64-cross && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-gnu
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc

# create a new empty shell project
RUN USER=root cargo new --bin ripleybot
WORKDIR /ripleybot

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release --target x86_64-unknown-linux-gnu
RUN rm -r src

# copy your source tree
COPY ./src ./src
RUN cat ./src/main.rs
RUN ls ./src

# build for release
RUN rm ./target/x86_64-unknown-linux-gnu/release/deps/RipleyPlanetWarsBot*
RUN cargo build --release --target x86_64-unknown-linux-gnu

# our final base
FROM debian:bullseye-slim
WORKDIR /ripleybot

# copy the build artifact from the build stage
COPY --from=builder /ripleybot/target/x86_64-unknown-linux-gnu/release/RipleyPlanetWarsBot /ripleybot/RipleyPlanetWarsBot

# set the startup command to run your binary
CMD ["/ripleybot/RipleyPlanetWarsBot"]
