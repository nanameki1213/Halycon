#!/bin/sh

QEMU=~/qemu-9.0.2/build/qemu-system-riscv64
mv $1 ./bin/disk

$QEMU \
  -M virt \
  -smp 1 \
  -bios /usr/lib/riscv64-linux-gnu/opensbi/generic/fw_jump.bin \
  -nographic -m 4G \
  -kernel bin/disk/hypervisor
