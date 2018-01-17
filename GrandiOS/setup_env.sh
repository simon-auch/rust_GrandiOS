curl https://sh.rustup.rs -sSf | sh
. $HOME/.cargo/env
rustup default nightly
rustup component add rust-src
cargo install xargo
