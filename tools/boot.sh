source ./environment

mkdir -p $DISK_IMG_DIR
mkdir -p $L1_DISK_IMG_DIR

$U_BOOT_DIR/tools/mkimage -A riscv -T script -C none -d $SOURCE_DIR/boot.script $DISK_IMG_DIR/boot.scr

$U_BOOT_DIR/tools/mkimage -A riscv -T script -C none -d $SOURCE_DIR/boot_L1hypervisor.script $L1_DISK_IMG_DIR/boot.scr

# device tree
$U_BOOT_DIR/scripts/dtc/dtc -I dts -O dtb -o $DISK_IMG_DIR/virt.dtb  $SOURCE_DIR/virt.dts
