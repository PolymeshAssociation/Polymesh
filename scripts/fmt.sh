#!/usr/bin/env bash
find . -name "Cargo.toml" -not -path "*/target/*" -not -path "*/external/*" -not -path "*/pallets/staking/fuzzer/*" -execdir bash -c "cargo fmt" \;
