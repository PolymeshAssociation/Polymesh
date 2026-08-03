#!/bin/sh
set -eu

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
  echo "Usage: $0 <precompile_address> [rpc_url]" >&2
  exit 1
fi

ADDRESS="$1"
RPC_URL="${2:-${RPC_URL:-http://127.0.0.1:8545}}"
ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
BIN_FILE="${BIN_FILE:-${ROOT_DIR}/precompiles/src/interfaces/FungibleAssetStub.bin}"

if [ ! -s "$BIN_FILE" ]; then
  echo "Bytecode artifact missing or empty: $BIN_FILE" >&2
  exit 1
fi

python3 - "$ADDRESS" "$RPC_URL" "$BIN_FILE" <<'PY'
import json
import pathlib
import sys
from urllib.request import Request, urlopen

address, rpc_url, bin_file = sys.argv[1:]

expected_hex = pathlib.Path(bin_file).read_bytes().hex().lower()
payload = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "eth_getCode",
    "params": [address, "latest"],
}

req = Request(rpc_url, data=json.dumps(payload).encode(), headers={"Content-Type": "application/json"})
with urlopen(req) as resp:
    result = json.load(resp)

if "error" in result:
    raise SystemExit(f"eth_getCode error: {result['error']}")

actual = result.get("result", "0x")
actual_hex = actual[2:].lower() if actual.startswith("0x") else actual.lower()
print(f"Address:      {address}")
print(f"RPC:          {rpc_url}")
print(f"Expected len: {len(expected_hex) // 2} bytes")
print(f"On-chain len: {len(actual_hex) // 2} bytes")

if actual_hex == expected_hex:
    print("MATCH: on-chain code equals FungibleAssetStub.bin")
    sys.exit(0)

if actual_hex == "":
    print("MISMATCH: on-chain code is empty (0x)")
else:
    print("MISMATCH: on-chain code differs from FungibleAssetStub.bin")
    print(f"Expected prefix: 0x{expected_hex[:32]}")
    print(f"Actual prefix:   0x{actual_hex[:32]}")

sys.exit(1)
PY
