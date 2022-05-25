#!/bin/sh
SCCACHE=`which sccache`

if [ ! -f "$SCCACHE" ]; then
	cargo install sccache
else
	echo "sccache installed"
fi

$SCCACHE -s
