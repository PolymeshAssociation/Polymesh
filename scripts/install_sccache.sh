#!/bin/sh

if [ ! -f "$(which sccache)" ]; then
	echo "install sccache"
	cargo install sccache
else
	echo "sccache already installed"
fi

sccache -s
