#!/bin/sh
# Full erase + flash — only needed when the partition table changes.
# For normal code updates, just use: cargo run / cargo run --release
espflash erase-flash && cargo espflash flash --monitor "$@"
