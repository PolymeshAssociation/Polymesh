#!/bin/sh
set -eu

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
  echo "Usage: $0 <precompile_address> [blockscout_api_url]" >&2
  exit 1
fi

ADDRESS="$1"
API_URL="${2:-${API_URL:-http://127.0.0.1:4001}}"
ROOT_DIR="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
SOURCE_FILE="${SOURCE_FILE:-${ROOT_DIR}/precompiles/src/interfaces/FungibleAssetStub.sol}"

if [ ! -f "$SOURCE_FILE" ]; then
  echo "Missing source file: $SOURCE_FILE" >&2
  exit 1
fi

python3 - "$ADDRESS" "$API_URL" "$SOURCE_FILE" <<'PY'
import json
import pathlib
import sys
from urllib.request import Request, urlopen

address, api_url, source_file = sys.argv[1:]
source = pathlib.Path(source_file).read_text()

body = {
    "compiler_version": "v0.8.33+commit.64118f21",
    "source_code": source,
    "is_optimization_enabled": True,
    "optimization_runs": 200,
    "contract_name": "FungibleAssetStub",
    "evm_version": "shanghai",
    "autodetect_constructor_args": False,
    "constructor_args": "",
    "license_type": "mit",
}

url = f"{api_url}/api/v2/smart-contracts/{address}/verification/via/flattened-code"
req = Request(url, data=json.dumps(body).encode(), headers={"Content-Type": "application/json"})

with urlopen(req) as resp:
    print(resp.read().decode())
PY
