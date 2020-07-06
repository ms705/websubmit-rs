#!/bin/bash

cargo run --bin noria-zk -- --deployment hello --clean
rm -r *.db

rm remove_user.txt
rm client_side_time_to_add_user.txt
rm write_time.txt
rm end_times.txt
rm start_times.txt
rm results
rm intervals.txt

RUST_BACKTRACE=1 cargo run -- -i hello
# &> run.txt

