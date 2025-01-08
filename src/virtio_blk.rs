use crate::virtio::*;

pub fn init_virtio_blk() {
    // 1. Reset the device.
    set_virtio_mmio(VIRTIO_MMIO_STATUS, 0x0);
    // 2. Set the ACKNOWLEDGE status bit
    set_virtio_mmio(VIRTIO_MMIO_STATUS, VIRTIO_MMIO_STATUS_ACKNOWLEDGE as u32);
    // 3. Set the DRIVER status bit
    let mut status = get_virtio_mmio(VIRTIO_MMIO_STATUS);
    status |= VIRTIO_MMIO_STATUS_DRIVER as u32;
    set_virtio_mmio(VIRTIO_MMIO_STATUS, status);
    // 4. Read device feature bits, and write the subset of feature bits understood
    //    by the OS and driver to the device.
    // TODO: setting feature bits

    // 5. Set the FEATURES_OK status bit.
    status = get_virtio_mmio(VIRTIO_MMIO_STATUS);
    status |= VIRTIO_MMIO_STATUS_FEATURES_OK as u32;
    set_virtio_mmio(VIRTIO_MMIO_STATUS, status);
    // 6. Re-read device status to ensure the FEATURES_OK bit is still set
    // TODO: setting feature bits

    // 7. Perform device-specific setup

    // 8. Set the DRIVER_OK status bit. At this point the device is "live".
    status = get_virtio_mmio(VIRTIO_MMIO_STATUS);
    status |= VIRTIO_MMIO_STATUS_DRIVER_OK as u32;
    set_virtio_mmio(VIRTIO_MMIO_STATUS, status);
}
