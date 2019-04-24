#!/usr/bin/env bash
script_dir=$(dirname $0)
set -e

pushd $script_dir/../polymesh_substrate 1>/dev/null
	# rustfmt all top-level, non-artifact `src` dirs, all of *.rs inside
	if [ -z "${VERBOSE-}" ] ; then
		find . -type d -name "src" -not -path "*/target/*" \
		       | xargs -i find {} -type f -name "*.rs" \
		       | xargs rustfmt +nightly --check 1>/dev/null || (echo "rustfmt FAIL" && false)

	else # rustfmt output not suppresed
		find . -type d -name "src" -not -path "*/target/*" \
		       | xargs -i find {} -type f -name "*.rs" \
		       | xargs rustfmt +nightly --check || (echo "rustfmt FAIL" && false)

	fi
	echo rustfmt OK
popd 1>/dev/null
