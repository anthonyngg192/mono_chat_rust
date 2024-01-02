#!/bin/sh

if [ -z "$TARGETARCH" ]; then
  :
else
  case "${TARGETARCH}" in
    "amd64")
      LINKER_NAME="x86_64-linux-gnu-gcc"
      LINKER_PACKAGE="gcc-x86-64-linux-gnu"
      BUILD_TARGET="x86_64-unknown-linux-gnu" ;;
    "arm64")
      LINKER_NAME="aarch64-linux-gnu-gcc"
      LINKER_PACKAGE="gcc-aarch64-linux-gnu"
      BUILD_TARGET="aarch64-unknown-linux-gnu" ;;
  esac
fi

tools() {
  apt-get install -y "${LINKER_PACKAGE}"
  rustup target add "${BUILD_TARGET}"
}

deps() {
  mkdir -p \
    crates/api/src \
    crates/core/src \
    crates/january/src \
    crates/socket/src \
    crates/voso/src 
  echo 'fn main() { panic!("stub"); }' |
    tee crates/api/src/main.rs |
    tee crates/voso/src/main.rs |
    tee crates/socket/src/main.rs |
    tee crates/january/src/main.rs
  echo '' |
    tee crates/core/src/lib.rs 
  
  if [ -z "$TARGETARCH" ]; then
    cargo build --locked --release
  else
    cargo build --locked --release --target "${BUILD_TARGET}"
  fi
}

apps() {
  touch -am \
    crates/core/src/lib.rs \
    crates/api/src/main.rs \
    crates/core/january/src/main.rs \f
    crates/core/voso/src/main.rs \
    crates/core/voso/socket/main.rs
  
  if [ -z "$TARGETARCH" ]; then
    cargo build --locked --release
  else
    cargo build --locked --release
    mv target _target && mv _target/"${BUILD_TARGET}" target
  fi
}

if [ -z "$TARGETARCH" ]; then
  :
else
  export RUSTFLAGS="-C linker=${LINKER_NAME}"
  export PKG_CONFIG_ALLOW_CROSS="1"
  export PKG_CONFIG_PATH="/usr/lib/pkgconfig:/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/lib/x86_64-linux-gnu/pkgconfig"
fi

"$@"
