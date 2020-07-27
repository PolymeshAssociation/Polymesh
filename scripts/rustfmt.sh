#!/usr/bin/env bash
if [ -z "$(find . -name "Cargo.toml" -not -path "*/target/*" -not -path "*/external/*" -execdir bash -c "cargo fmt -- --check" \;)" ]; then
	echo "rustfmt ok"
	exit 0
else
	echo "rustfmt error"
	exit 1
fi
