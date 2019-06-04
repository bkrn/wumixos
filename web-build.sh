#! /usr/bin/env sh
set -e

# mkdir -p "static/media"
# Fetch, checksum, and decrypt UMIX OS if necessary
if [ ! -f "static/media/umix_os.um" ]; then
    cargo run --release --bin decrypt --features "decrypt" -- "static/media/umix_os.um"
fi

# Fetch and checksum sandmark is necessary
if [ ! -f "static/media/sandmark.umz" ]; then
    curl -o "static/media/sandmark.umz" "http://www.boundvariable.org/sandmark.umz"
    echo 'c2ee087aa661e81407fbcf0d9d7e503aff9b268e static/media/sandmark.umz' | sha1sum -c -
fi

# Build the VM WASM and js wrapper and move to the static
# files directory
mkdir -p static/machine
cargo web build --release --bin machine --features "web"
cp target/wasm32-unknown-unknown/release/machine.js static/machine/machine.js
cp target/wasm32-unknown-unknown/release/machine.wasm static/machine/machine.wasm

# Either burn a development server or build and move the
# view WASM depending on the first argument being "run"
if [ "${1}" = "run" ]; then
    cargo-web start --release --bin web --features "web"
else
    cargo-web build --release --bin web --features "web"
    cp target/wasm32-unknown-unknown/release/web.js static
    cp target/wasm32-unknown-unknown/release/web.wasm static
fi