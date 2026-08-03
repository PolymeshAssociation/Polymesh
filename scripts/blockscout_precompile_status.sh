#!/bin/sh
set -eu

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
  echo "Usage: $0 <precompile_address> [blockscout_api_url]" >&2
  exit 1
fi

ADDRESS="$1"
API_URL="${2:-${API_URL:-http://127.0.0.1:4001}}"

python3 - "$ADDRESS" "$API_URL" <<'PY'
import json
import sys
from urllib.request import Request, urlopen

address, api_url = sys.argv[1:]
url = f"{api_url}/api/v2/smart-contracts/{address}"

req = Request(url)
with urlopen(req) as resp:
    data = json.load(resp)

print(f"address: {address}")
print(f"name: {data.get('name')}")
print(f"is_verified: {data.get('is_verified')}")
print(f"is_partially_verified: {data.get('is_partially_verified')}")
print(f"is_fully_verified: {data.get('is_fully_verified')}")
print(f"verified_twin_address_hash: {data.get('verified_twin_address_hash')}")
print(f"abi_len: {len(data['abi']) if data.get('abi') else 0}")
PY
