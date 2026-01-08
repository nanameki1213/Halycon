mkdir -p ../bin/disk
mkdir -p ../bin/l1_disk

cd u-boot
make qemu-riscv64_defconfig
CROSS_COMPILE=riscv64-linux-gnu- make -j$(nproc)
mv u-boot ../../bin/

make qemu-riscv64_smode_defconfig
CROSS_COMPILE=riscv64-linux-gnu- make -j$(nproc)
mv u-boot.bin ../../bin/disk/
cp ../bin/disk/u-boot.bin ../bin/l1_disk
