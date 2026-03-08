#!/bin/sh
# Runner script for `cargo run` / `cargo run --release`.
# Flashes bootloader + partition table + app together so they always match.
ELF="$1"
PROFILE_DIR=$(dirname "$ELF")

BOOTLOADER=$(find "$PROFILE_DIR/build" -name "bootloader.bin" 2>/dev/null | head -1)
PART_TABLE=$(find "$PROFILE_DIR/build" -name "partition-table.bin" 2>/dev/null | head -1)

if [ -n "$BOOTLOADER" ] && [ -n "$PART_TABLE" ]; then
    espflash flash --bootloader "$BOOTLOADER" --partition-table "$PART_TABLE" --monitor "$ELF"
else
    echo "Warning: could not find bootloader/partition-table, flashing app only"
    espflash flash --monitor "$ELF"
fi
