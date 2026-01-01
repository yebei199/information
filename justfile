set shell := ["fish", "-c"]
set dotenv-load := true
set export := true

check:
    cargo check

clean:
    cargo sweep --time 0 .

release:
    cargo build --release

musl_release:
    cargo build --release --target x86_64-unknown-linux-musl
