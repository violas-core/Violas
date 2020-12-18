cd language/stdlib/ && cargo run --release 
cd ../.. && cargo build --release --bin diem-node --bin move-build 
cd testsuite/diem-swarm && cargo build --release 
cd ../cli && cargo build --release
cd ../../target/release && strip *

