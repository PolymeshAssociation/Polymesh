#!/usr/bin/env bash
BRANCH=${1:-test}
COMMIT=`echo ${2:-hash} | cut -c-10`
export VERSION="$COMMIT"

if [[ "x$BRANCH" != "xdevelop" ]]; then
	export VERSION=`grep ^version ./Cargo.toml | head -1 | cut -d"=" -f2 | sed 's/[^a-zA-Z0-9\.]//g'`
fi
echo "$VERSION"
