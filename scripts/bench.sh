cargo build --release --features=runtime-benchmarks && \
./target/release/polymesh benchmark -p=* -e=* -s 200 -r 10 --execution Wasm --wasm-execution Compiled --output
