cargo build --release --features=runtime-benchmarks && \
./target/release/polymesh benchmark -p=pallet_staking -e=* -s 20 -r 1 --execution Wasm --wasm-execution Compiled --output
