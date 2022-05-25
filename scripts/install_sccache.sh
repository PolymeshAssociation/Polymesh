#!/bin/sh

if [ ! -f "$(which sccache)" ]; then
	unset RUSTC_WRAPPER
	echo "install sccache"
	cargo install sccache
else
	echo "sccache already installed"
fi

sccache -s
