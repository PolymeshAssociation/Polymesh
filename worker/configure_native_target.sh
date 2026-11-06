#!/usr/bin/env bash

configure_native_target() {
  local target="$1"
  if [[ "$target" != "aarch64-unknown-linux-gnu" ]]; then
    return 0
  fi

  if [[ -z "${CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER:-}" ]]; then
    if command -v aarch64-linux-gnu-gcc >/dev/null 2>&1; then
      CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
    elif [[ "$(uname -m)" == "aarch64" ]] && command -v gcc >/dev/null 2>&1; then
      CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=gcc
    else
      echo "AArch64 cross-build requires aarch64-linux-gnu-gcc or an explicit CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER" >&2
      return 1
    fi
  fi

  export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER
  export CC_aarch64_unknown_linux_gnu="${CC_aarch64_unknown_linux_gnu:-$CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER}"
}
