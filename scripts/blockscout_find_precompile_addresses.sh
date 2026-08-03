#!/bin/sh
set -eu

RPC_URL="${RPC_URL:-http://127.0.0.1:8545}"
FROM_BLOCK="${FROM_BLOCK:-0x0}"
TO_BLOCK="${TO_BLOCK:-latest}"
TRANSFER_TOPIC="${TRANSFER_TOPIC:-0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef}"

python3 - "$RPC_URL" "$FROM_BLOCK" "$TO_BLOCK" "$TRANSFER_TOPIC" <<'PY'
import json
import sys
from urllib.request import Request, urlopen

rpc_url, from_block, to_block, transfer_topic = sys.argv[1:]

payload = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "eth_getLogs",
    "params": [{
        "fromBlock": from_block,
        "toBlock": to_block,
        "topics": [transfer_topic],
    }],
}

req = Request(rpc_url, data=json.dumps(payload).encode(), headers={"Content-Type": "application/json"})
with urlopen(req) as resp:
    result = json.load(resp)

if "error" in result:
    raise SystemExit(f"eth_getLogs error: {result['error']}")

logs = result.get("result", [])
addresses = []
seen = set()
for log in logs:
    addr = log.get("address", "").lower()
    if addr and addr not in seen:
        seen.add(addr)
        addresses.append(addr)

print(f"RPC_URL={rpc_url}")
print(f"Transfer logs: {len(logs)}")
print(f"Unique addresses: {len(addresses)}")
for addr in addresses:
    print(addr)
PY
