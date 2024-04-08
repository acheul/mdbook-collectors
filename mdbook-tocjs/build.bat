@echo off
python ../_build/condense_and_copy.py --src_path="./tocjs" --build_path="./src/blobs.rs"
cargo build --release
REM cargo install --path=.