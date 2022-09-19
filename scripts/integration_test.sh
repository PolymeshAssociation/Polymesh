#!/usr/bin/env bash
set -e

script_dir=$(dirname $0)

INTEGRATION_TEST=true $script_dir/test.sh
