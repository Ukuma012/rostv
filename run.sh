#!/bin/bash
set -xue

QEMU=qemu-system-riscv64
KERNEL=target/riscv64imac-unknown-none-elf/debug/rost_riscv

cargo build
$QEMU -machine virt -bios default --no-reboot -serial stdio \
      -device virtio-gpu-device \
      -kernel $KERNEL