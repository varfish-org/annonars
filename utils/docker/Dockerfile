# syntax=docker/dockerfile:1.3

# Based on https://levelup.gitconnected.com/1940db638a6c
#
# We don't do cross compilation at the moment but build the dependencies first
# anyway to get the ability to increment.
#
#

# ---------------------------------------------------------------------------
# Builder
# ---------------------------------------------------------------------------

# Pinning Rust version for now because of this issue:
#
# - https://github.com/rust-lang/rust/issues/95926
FROM rust:1-bookworm AS builder

# Build dependencies first.
#
# Install dependencies for compilation of C code (e.g., rocksdb).
RUN apt-get update && \
    apt-get install -y clang
# Add the needed Cargo components.
RUN rustup component add rustfmt
# Install build dependency `protoc`.
COPY utils/install-protoc.sh /tmp
RUN CMAKE_INSTALL_PREFIX=/usr/local bash /tmp/install-protoc.sh

# Now for the two-step building.
#
# Set initial workdir.
WORKDIR /usr/src
# Create blank project.
RUN USER=root cargo new annonars
# We want dependencies cached, so copy those first.
COPY Cargo.toml Cargo.lock /usr/src/annonars/
RUN touch /usr/src/annonars/src/lib.rs
# Set the working directory.
WORKDIR /usr/src/annonars
# This is a dummy build to get the dependencies cached.
RUN cargo build --release
#
# Now copy in the rest of the sources.
COPY build.rs /usr/src/annonars/
COPY src /usr/src/annonars/src/
COPY protos /usr/src/annonars/protos/
COPY utils/alpine-linker-script.sh /usr/src/annonars/utils/
RUN chmod a+rx /usr/src/annonars/utils/alpine-linker-script.sh
COPY .cargo /usr/src/annonars/.cargo/
## Touch main.rs to prevent cached release build.
RUN touch /usr/src/annonars/src/main.rs
# This is the actual application build.
RUN cargo build --release

# ---------------------------------------------------------------------------
# Runtime
# ---------------------------------------------------------------------------

FROM debian:bookworm-slim AS runtime

# Copy application binary from builder image
COPY --from=builder \
    /usr/src/annonars/target/release/annonars \
    /usr/local/bin

# Install dependencies (and cleanup afterwards)
RUN apt-get update && \
    apt-get install -y libsqlite3-0 && \
    apt-get clean autoclean && \
    apt-get autoremove --yes && \
    rm -rf /var/lib/{apt,dpkg,cache,log}

# Copy the entrypoint script and make it executable.
COPY utils/docker/entrypoint.sh /
RUN chmod a+rx /entrypoint.sh

# Set the entrypoint.
ENTRYPOINT ["/bin/bash", "/entrypoint.sh"]
# Set port to expose
EXPOSE 8080
