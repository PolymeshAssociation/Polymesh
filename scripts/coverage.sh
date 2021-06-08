#!/bin/bash

function run_tests() {
    RUSTFLAGS="-Zinstrument-coverage -Clink-dead-code" \
    LLVM_PROFILE_FILE="json5format-%m.profraw" \
    BUILD_DUMMY_WASM_BINARY=1 \
    cargo test --tests \
        --package pallet-staking \
        --package pallet-group \
        --package pallet-sudo \
        --package polymesh-primitives \
        --package node-rpc-runtime-api \
        --package pallet-transaction-payment \
        --package polymesh-runtime-tests \
        --package pallet-balances:0.1.0 \
        --features default_identity $*
}

function get_tests_filenames() {
    for file in \
        $( run_tests --no-run --message-format=json \
            | jq -r "select(.profile.test == true) | .filenames[]" \
            | grep -v dSYM - \
        );
    do
        printf "%s %s " -object $file;
    done
}

run_tests

cargo profdata -- merge -sparse $(find . -name 'json5format-*.profraw') -o json5format.profdata

if [[ -v CIRCLECI ]]; then
    cargo cov -- export \
    $( get_tests_filenames ) \
    --format='lcov' \
    --instr-profile=json5format.profdata \
    --ignore-filename-regex='/.cargo/registry/' \
    --ignore-filename-regex='/.cargo/git/' \
    --ignore-filename-regex='/target/debug/' \
    --ignore-filename-regex='/tests/' \
    --ignore-filename-regex='bin/' \
    --ignore-filename-regex='contracts/' \
    --ignore-filename-regex='rpc/' \
    --ignore-filename-regex='/rustc/' > coverage.txt

    bash <(curl -s https://codecov.io/bash)
else
    cargo cov -- report \
    $( get_tests_filenames ) \
    --instr-profile=json5format.profdata \
    --use-color \
    --ignore-filename-regex='/.cargo/registry/' \
    --ignore-filename-regex='/.cargo/git/' \
    --ignore-filename-regex='/target/debug/' \
    --ignore-filename-regex='/tests/' \
    --ignore-filename-regex='bin/' \
    --ignore-filename-regex='contracts/' \
    --ignore-filename-regex='rpc/' \
    --ignore-filename-regex='/rustc/' \
    --summary-only

    find . -name '*.profraw' -delete
fi
