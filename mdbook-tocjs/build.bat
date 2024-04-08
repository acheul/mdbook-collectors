@echo off
python tocjs/build/condense_and_copy.py
cargo build --release
cargo install --path=.