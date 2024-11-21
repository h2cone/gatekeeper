#!/bin/sh

RUST_LOG=INFO ./target/release/gatekeeper -d -c ./conf.yaml \
--ba 0.0.0.0:8008 \
--hcf 30 \
--ua 127.0.0.1:3000 \
--ua 127.0.0.1:8090

cat ./conf.yaml
