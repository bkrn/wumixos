# README

## Sandmark

`cat sandmark.umz | cargo run --bin term --release`

## Codex Decryption

Download, verify, and decrypt UMIX OS

Defaults to writing the binary to `./umix_os.um` but
you can provide a different target file as an argument

`cargo run --release --bin decrypt`

## Launch umix OS in terminal

Assuming you have run the Codex Decryption application with defaults

`cat umix_os.um | cargo run --bin term --release`

Passing in arguments to run on launch

`cat umix_os.um | cargo run --bin term --release -- "guest" "mail"`

## Launch VM in web browser locally

```sh 
# The VM is run outside of the main thread in a webworker so the VM and View are
# built separately 
cargo web build --bin machine --target wasm32-unknown-unknown --release --features "web" && \
cp target/wasm32-unknown-unknown/release/machine.* static/machine && \
cargo-web start --release --bin web --features "web"```