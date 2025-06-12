FROM rust:1.83.0-bullseye as builder

# create a new empty shell project
RUN USER=root cargo new --bin ripleybot
WORKDIR /ripleybot

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm -r src

# copy your source tree
COPY ./src ./src
RUN cat ./src/main.rs
RUN ls ./src

# build for release
RUN rm ./target/release/deps/RipleyPlanetWarsBot*
RUN cargo build --release

# our final base
FROM debian:bullseye-slim
WORKDIR /ripleybot

# copy the build artifact from the build stage
COPY --from=builder /ripleybot/target/release/RipleyPlanetWarsBot /ripleybot/RipleyPlanetWarsBot

# set the startup command to run your binary
CMD ["/ripleybot/RipleyPlanetWarsBot"]
