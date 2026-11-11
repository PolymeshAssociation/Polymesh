#!/usr/bin/env python3
"""Fail if the worker differential log contains incomplete or divergent results."""

from collections import Counter
import json
from pathlib import Path
import sys


def main() -> int:
    if len(sys.argv) != 2:
        print(f"usage: {sys.argv[0]} <worker-diff.jsonl>", file=sys.stderr)
        return 2

    path = Path(sys.argv[1])
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as error:
        print(f"cannot read worker differential log {path}: {error}", file=sys.stderr)
        return 2

    records = []
    failures = []
    for line_number, line in enumerate(lines, start=1):
        if not line.strip():
            continue
        try:
            record = json.loads(line)
        except json.JSONDecodeError as error:
            failures.append(f"line {line_number}: invalid JSON: {error}")
            continue
        if not isinstance(record, dict):
            failures.append(f"line {line_number}: record is not a JSON object")
            continue

        records.append(record)
        status = record.get("status")
        expected = record.get("expected_backends")
        backend_results = record.get("backends")
        if status != "match":
            failures.append(
                f"line {line_number}: status={status!r}, "
                f"protocol={record.get('protocol')}, request={record.get('request_hash')}"
            )
        if not isinstance(expected, list) or len(expected) < 2:
            failures.append(f"line {line_number}: fewer than two expected backends")
        if not isinstance(backend_results, list):
            failures.append(f"line {line_number}: missing backend results")
            continue

        loaded = {
            backend.get("backend")
            for backend in backend_results
            if isinstance(backend, dict)
        }
        if isinstance(expected, list):
            missing = set(expected) - loaded
            if missing:
                failures.append(f"line {line_number}: missing backend results: {sorted(missing)}")
        if record.get("response_match") is not True:
            failures.append(f"line {line_number}: response_match is not true")
        if record.get("context_match") is not True:
            failures.append(f"line {line_number}: context_match is not true")

    if not records:
        failures.append("worker differential log contains no records")

    statuses = Counter(record.get("status", "<missing>") for record in records)
    print(f"Checked {len(records)} worker differential records: {dict(statuses)}")
    if failures:
        print(f"Found {len(failures)} worker differential failure(s):", file=sys.stderr)
        for failure in failures[:50]:
            print(f"- {failure}", file=sys.stderr)
        if len(failures) > 50:
            print(f"- ... {len(failures) - 50} additional failure(s)", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
