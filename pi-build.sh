#!/bin/bash

docker build ./pi -t "pimesh" && \
docker run --rm -it \
	-v "$(pwd)":/home/rust/src pimesh \
	cargo build --target=aarch64-unknown-linux-gnu --release
