#!/bin/sh

QEMU=~/qemu-9.0.2/build/qemu-system-riscv64
mv $1 ./bin/disk

$QEMU \
  -M virt \
  -smp 1 -bios ~/u-boot/u-boot.bin \
  -nographic -m 2G \
  -device virtio-blk-device,drive=disk \
  -drive file=fat:rw:bin/disk/,format=raw,if=none,media=disk,id=disk \
  -global virtio-mmio.force-legacy=false \
  -cdrom ~/alpine-virt-3.20.2-aarch64.iso
  
  # -kernel bin/disk/hypervisor \
  # -serial mon:stdio \
  # --no-reboot \
  # -device virtio-net-device,netdev=usernet,bus=virtio-mmio-bus.0 \
  # -object filter-dump,id=f1,netdev=usernet,file=dump.dat
  # -netdev user,id=usernet,net=192.168.11.13/24 \
  # -device virtio-blk-device,drive=disk \
  # -drive file=fat:rw:bin/,format=raw,if=none,media=disk,id=disk \
