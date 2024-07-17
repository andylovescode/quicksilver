echo Clippy
cargo clippy --fix --allow-dirty

echo Format
cargo fmt

echo Test
cargo test
