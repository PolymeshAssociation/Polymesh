#!/usr/bin/env bash
if cargo fmt -- --check; then
	echo "rustfmt ok"
	exit 0
else
	echo "rustfmt error"
	exit 1
fi
