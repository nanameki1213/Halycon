mkdir -p ../bin/disk

cd u-boot
make qemu-riscv64_defconfig
CROSS_COMPILE=riscv64-linux-gnu- make -j$(nproc)
mv u-boot ../../bin/disk/

make qemu-riscv64_smode_defconfig
CROSS_COMPILE=riscv64-linux-gnu- make -j$(nproc)
mv u-boot.bin ../../bin/
