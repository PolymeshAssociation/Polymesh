#!/usr/bin/env bash
if [ -z "$(find . -name "Cargo.toml" -not -path "*/target/*" -execdir bash -c "cargo +nightly fmt -- --check" \;)" ]; then
	echo "rustfmt ok"
	exit 0
else
	echo "rustfmt error"
	exit 1
fi
