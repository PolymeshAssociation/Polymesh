#!/usr/bin/env bash
find . -name "Cargo.toml" -not -path "*/target/*" -not -path "*/external/*" -execdir bash -c "cargo fmt" \;
