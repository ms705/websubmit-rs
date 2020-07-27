#!/bin/bash

cd /Users/eleonorakiziv/rust/noria/noria
cargo run --bin noria-zk -- --deployment hello --clean
rm -r *.db
cd /Users/eleonorakiziv/rust/websubmit-rs/websubmit-rs

rm client_side_time_to_add_user.txt
rm write_time.txt
rm end_times.txt
rm start_times.txt
rm results.txt
rm intervals.txt
rm un_times.txt
rm re_times.txt
rm imported_data.txt

RUST_BACKTRACE=full cargo run -- -i hello
# &> run.txt

