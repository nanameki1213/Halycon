source ./environment

$U_BOOT_DIR/tools/mkimage -A riscv -T script -C none -d ../scripts/boot.script ../$DISK_IMG_DIR/boot.scr
