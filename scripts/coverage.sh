# For someone reason, WASM toolchain is not detected by the next command
# Therefore, building wasm binaries using this command and skipping in next
cargo check

RUSTFLAGS="-Zinstrument-coverage" \
LLVM_PROFILE_FILE="json5format-%m.profraw" \
SKIP_WASM_BUILD=1 \
cargo test -j 1 --tests \
    --package pallet-staking \
    --package pallet-group \
    --package pallet-sudo \
    --package polymesh-primitives \
    --package node-rpc-runtime-api \
    --package pallet-transaction-payment \
    --package polymesh-runtime-tests \
    --package pallet-balances:0.1.0 \
    --features default_identity

cargo profdata -- merge \
    -sparse $(find . -name 'json5format-*.profraw') -o json5format.profdata

if [[ -v CIRCLECI ]]; then
    cargo cov -- export \
    $( \
        for file in \
            $( \
            RUSTFLAGS="-Zinstrument-coverage" SKIP_WASM_BUILD=1 \
                cargo test --tests \
                    --package pallet-staking \
                    --package pallet-group \
                    --package pallet-sudo \
                    --package polymesh-primitives \
                    --package node-rpc-runtime-api \
                    --package pallet-transaction-payment \
                    --package polymesh-runtime-tests \
                    --package pallet-balances:0.1.0 \
                    --features default_identity \
                    --no-run --message-format=json \
                | jq -r "select(.profile.test == true) | .filenames[]" \
                | grep -v dSYM - \
            ); \
        do \
            printf "%s %s " -object $file; \
        done \
    ) \
    --format='lcov' \
    --instr-profile=json5format.profdata \
    --ignore-filename-regex='/.cargo/registry/' \
    --ignore-filename-regex='/.cargo/git/' \
    --ignore-filename-regex='/target/debug/' \
    --ignore-filename-regex='/tests/' \
    --ignore-filename-regex='/bin/' \
    --ignore-filename-regex='/rustc/' > coverage.txt

    bash <(curl -s https://codecov.io/bash)
else
    cargo cov -- report \
    $( \
        for file in \
            $( \
            RUSTFLAGS="-Zinstrument-coverage" SKIP_WASM_BUILD=1 \
                cargo test --tests \
                    --package pallet-staking \
                    --package pallet-group \
                    --package pallet-sudo \
                    --package polymesh-primitives \
                    --package node-rpc-runtime-api \
                    --package pallet-transaction-payment \
                    --package polymesh-runtime-tests \
                    --package pallet-balances:0.1.0 \
                    --features default_identity \
                    --no-run --message-format=json \
                | jq -r "select(.profile.test == true) | .filenames[]" \
                | grep -v dSYM - \
            ); \
        do \
            printf "%s %s " -object $file; \
        done \
    ) \
    --instr-profile=json5format.profdata \
    --use-color \
    --ignore-filename-regex='/.cargo/registry/' \
    --ignore-filename-regex='/.cargo/git/' \
    --ignore-filename-regex='/target/debug/' \
    --ignore-filename-regex='/tests/' \
    --ignore-filename-regex='/bin/' \
    --ignore-filename-regex='/rustc/' \
    --summary-only

    find . -name '*.profraw' -delete
fi
