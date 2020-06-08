#!/bin/bash

cargo run --bin noria-zk -- --deployment $1 --clean
rm -r *.db
RUST_BACKTRACE=1 cargo run -- -i hello &> run.txt

