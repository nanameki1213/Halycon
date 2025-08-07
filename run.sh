#!/bin/sh

QEMU=~/qemu/build/qemu-system-riscv64
mv $1 ./bin/disk

$QEMU \
  -M virt \
  -smp 1 \
  -bios ./bin/disk/u-boot \
  -nographic -m 2G \
  -device virtio-blk-device,drive=drive0 \
  -drive file=fat:rw:bin/disk/,format=raw,if=none,media=disk,id=drive0 \
  -device virtio-blk-device,drive=drive1,bus=virtio-mmio-bus.0 \
  -drive file=./bin/u-boot.bin,if=none,format=raw,id=drive1 \
  -device virtio-blk-device,drive=drive2,bus=virtio-mmio-bus.1 \
  -drive file=./bin/virt.dtb,if=none,format=raw,id=drive2 \
  -device virtio-blk-device,drive=drive3,bus=virtio-mmio-bus.2 \
  -drive file=./disk.img,if=none,format=raw,id=drive3 \
  -global virtio-mmio.force-legacy=false \
  -D logfile.log -d in_asm,int \
  -s -S \

  # --trace events=./trace-events,file=trace.log \
  # -kernel bin/disk/hypervisor \
  # -serial mon:stdio \
  # --no-reboot \
  # -device virtio-net-device,netdev=usernet,bus=virtio-mmio-bus.0 \
  # -object filter-dump,id=f1,netdev=usernet,file=dump.dat
  # -netdev user,id=usernet,net=192.168.11.13/24 \
  # -device virtio-blk-device,drive=disk \
  # -drive file=fat:rw:bin/,format=raw,if=none,media=disk,id=disk \
