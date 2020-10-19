cargo build --release --features=runtime-benchmarks && \
./target/release/polymesh benchmark -p=pallet_identity -e=* -s 200 -r 10 --wasm-execution Compiled --execution Wasm --output
