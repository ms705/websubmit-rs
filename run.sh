#!/bin/bash

cargo run --bin noria-zk -- --deployment hello --clean
rm -r *.db
rm write_time.txt
RUST_BACKTRACE=1 cargo run --release -- -i hello
# &> run.txt

