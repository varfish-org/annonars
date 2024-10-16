# syntax=docker/dockerfile:1.3

# Based on https://levelup.gitconnected.com/1940db638a6c
#
# We don't do cross compilation at the moment but build the dependencies first
# anyway to get the ability to increment.

# ---------------------------------------------------------------------------
# Builder
# ---------------------------------------------------------------------------

# Use ubuntu:noble as the base image
FROM ubuntu:noble AS builder

# Install Rust toolchain and dependencies for compilation of C code (e.g., rocksdb)
RUN apt-get update && \
    apt-get install -y unzip wget curl build-essential clang librocksdb-dev libsqlite3-dev && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    . $HOME/.cargo/env && \
    rustup component add rustfmt

# Install build dependency `protoc`.
COPY utils/install-protoc.sh /tmp
RUN PREFIX=/usr/local bash /tmp/install-protoc.sh

# Now for the two-step building.
#
# Set initial workdir.
WORKDIR /usr/src
# Create blank project.
RUN USER=root /root/.cargo/bin/cargo new annonars
# We want dependencies cached, so copy those first.
COPY Cargo.toml Cargo.lock /usr/src/annonars/
RUN touch /usr/src/annonars/src/lib.rs
# Set the working directory.
WORKDIR /usr/src/annonars

# Copy in the rest of the sources.
COPY build.rs Cargo.toml Cargo.lock /usr/src/annonars/
COPY src /usr/src/annonars/src/
COPY protos /usr/src/annonars/protos/
COPY utils/alpine-linker-script.sh /usr/src/annonars/utils/
RUN chmod a+rx /usr/src/annonars/utils/alpine-linker-script.sh
COPY .cargo /usr/src/annonars/.cargo/

# Touch main.rs to prevent cached release build.
RUN touch /usr/src/annonars/src/main.rs

# Build the application
RUN /root/.cargo/bin/cargo build --release

# ---------------------------------------------------------------------------
# Runtime
# ---------------------------------------------------------------------------

FROM ubuntu:noble AS runtime

# Install dependencies (and cleanup afterward)
RUN apt-get update && \
    apt-get install -y librocksdb8.9 libsqlite3-0 && \
    apt-get clean autoclean && \
    apt-get autoremove --yes && \
    rm -rf /var/lib/{apt,dpkg,cache,log}

# Copy application binary from builder image
COPY --from=builder \
    /usr/src/annonars/target/release/annonars \
    /usr/local/bin

# Copy the entrypoint script and make it executable.
COPY utils/docker/entrypoint.sh /
RUN chmod a+rx /entrypoint.sh

# Set the entrypoint.
ENTRYPOINT ["/bin/bash", "/entrypoint.sh"]

# Expose the application port
EXPOSE 8080
