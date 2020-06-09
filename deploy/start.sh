#!/bin/sh

rm -rf ~/.local/share/polymesh
./target/release/polymesh --dev --unsafe-ws-external
