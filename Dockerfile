# Build Stage
FROM --platform="${BUILDPLATFORM}" rust:1.73.0
USER 0:0
WORKDIR /home/rust/src

ARG TARGETARCH

# Install build requirements
RUN dpkg --add-architecture "${TARGETARCH}"
RUN apt-get update && \
    apt-get install -y \
    make \
    pkg-config \
    libssl-dev:"${TARGETARCH}"
COPY scripts/build-image-layer.sh /tmp/
RUN sh /tmp/build-image-layer.sh tools

# Build all dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates/api/Cargo.toml ./crates/api/
COPY crates/core/Cargo.toml ./crates/core/
COPY crates/january/Cargo.toml ./crates/january/
COPY crates/socket/Cargo.toml ./crates/socket/
RUN sh /tmp/build-image-layer.sh deps

# Build all apps
COPY crates ./crates
RUN sh /tmp/build-image-layer.sh apps
