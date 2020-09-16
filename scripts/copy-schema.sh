#!/usr/bin/env bash

# Copies schema to the clipboard
# To run, call it from root dire of the repo: ./scripts/copy-schema.sh
# Must have jq and xclip installed. Works on Linux systems
# There are xlicp alternatives for mac and windows available
cat polymesh_schema.json | jq '.types' | xclip -sel clip
