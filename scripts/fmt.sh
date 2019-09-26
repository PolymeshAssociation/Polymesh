#!/usr/bin/env bash
script_dir=$(dirname $0)
set -e

pushd $script_dir/../polymesh_substrate 1>/dev/null
	find . -type d -name "src" -not -path "*/target/*" \
			| xargs -i find {} -type f -name "*.rs" \
			| xargs rustfmt +nightly || (echo "rustfmt FAIL" && false)

	echo rustfmt successfull
popd 1>/dev/null
