#!/usr/bin/env bash
set -euo pipefail
VERSIONS=("${@:-v1}")

worker_tools() {
	if [ -n "${POLYMESH_WORKER_TOOLS:-}" ]; then
		"$POLYMESH_WORKER_TOOLS" "$@"
	else
		cargo run --locked -r -p polymesh-worker-tools -- "$@"
	fi
}

for VERSION in "${VERSIONS[@]}"; do
	case "$VERSION" in
		v0) PROTOCOL_VERSION="0.1.0" ;;
		v1) PROTOCOL_VERSION="1.0.0" ;;
		*)
			echo "Unknown DART module version: $VERSION" >&2
			exit 1
			;;
	esac

	./build_polkavm.sh "$VERSION"
	./build_polkavm.sh "$VERSION" "testing"
	./build_wasm.sh "$VERSION"
	./build_wasm.sh "$VERSION" "testing"

	crate="polymesh-worker-protocol-dart-$VERSION"
	module_dir="modules/dart/$VERSION"
	config="$module_dir/$crate.config.scale"
	worker_tools build-release-config \
		--protocol-version "$PROTOCOL_VERSION" \
		--polkavm "$module_dir/$crate.polkavm.zst" \
		--wasm "$module_dir/$crate.wasm.zst" \
		--output "$config"
	worker_tools export-config-json \
		--config "$config" \
		--output "$module_dir/$crate.config.json"
done
