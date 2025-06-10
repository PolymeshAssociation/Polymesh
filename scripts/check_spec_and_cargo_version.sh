#!/usr/bin/env bash
SPEC_COUNT=`grep -h -r ' spec_version: ' pallets/runtime/*/src/runtime.rs | sort -u | wc -l`
if [ "$SPEC_COUNT" = "1" ]; then
	echo "Spec count ok"
else
	echo "Multiple Spec versions."
	exit 1
fi
POLYMESH_VERSION=`grep -h -r ' spec_version: ' pallets/runtime/*/src/runtime.rs | sort -u | sed -e 's/.*spec_version: \([0-9]\+\)_[0]*\([0-9]\+\)_[0]*\([0-9]\+\)[0-9],/\1.\2.\3/g'`

NODE_PKG_VERSION=`cargo info -q --color never polymesh | grep "^version:" | sed -e 's/version: \([0-9a-z.-]*\) .*/\1/g'`

if echo "$NODE_PKG_VERSION" | grep -q $POLYMESH_VERSION; then
	echo "Spec version matches Polymesh version."
	exit 0
else
	echo "Spec version doesn't match Polymesh version."
	echo "Version from spec_version=$POLYMESH_VERSION"
	echo "Version from node package=$NODE_PKG_VERSION"
	exit 1
fi
