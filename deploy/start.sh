#!/bin/sh

./target/release/polymesh --dev --unsafe-ws-external &
sleep 3
cd ./scripts/cli && npm run dev
