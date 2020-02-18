#!/usr/bin/env bash
find . -name "Cargo.toml" -not -path "*/target/*" -execdir bash -c "cargo +nightly fmt" \;
