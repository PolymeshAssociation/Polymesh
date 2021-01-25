# RUSTFLAGS="-Zinstrument-coverage" \
# LLVM_PROFILE_FILE="json5format-%m.profraw" \
# SKIP_WASM_BUILD=1 \
# cargo test --tests \
#     --package polymesh-runtime-tests \
#     --features default_identity \
#     --package pallet-staking \
#     --package pallet-balances:0.1.0 \
#     --package polymesh-primitives \
#     --package pallet-transaction-payment

# cargo profdata -- merge \
#     -sparse $(find . -name 'json5format-*.profraw') -o json5format.profdata

if [[ -z "${DEPLOY_ENV}" ]]; then
    cargo cov -- export \
    $( \
        for file in \
            $( \
            RUSTFLAGS="-Zinstrument-coverage" SKIP_WASM_BUILD=1 \
                cargo test --tests \
                    --package polymesh-runtime-tests \
                    --features default_identity \
                    --package pallet-staking \
                    --package pallet-balances:0.1.0 \
                    --package polymesh-primitives \
                    --package pallet-transaction-payment \
                    --no-run --message-format=json \
                | jq -r "select(.profile.test == true) | .filenames[]" \
                | grep -v dSYM - \
            ); \
        do \
            printf "%s %s " -object $file; \
        done \
    ) \
    --instr-profile=json5format.profdata \
    --ignore-filename-regex='/.cargo/registry/' \
    --ignore-filename-regex='/.cargo/git/' \
    --ignore-filename-regex='/target/debug/' \
    --ignore-filename-regex='/tests/' \
    --ignore-filename-regex='/rustc/' > coverage.json

    bash <(curl -s https://codecov.io/bash)
else
    cargo cov -- report \
    $( \
        for file in \
            $( \
            RUSTFLAGS="-Zinstrument-coverage" SKIP_WASM_BUILD=1 \
                cargo test --tests \
                    --package polymesh-runtime-tests \
                    --features default_identity \
                    --package pallet-staking \
                    --package pallet-balances:0.1.0 \
                    --package polymesh-primitives \
                    --package pallet-transaction-payment \
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
    --ignore-filename-regex='/rustc/' \
    --summary-only

    find . -name '*.profraw' -delete
fi
