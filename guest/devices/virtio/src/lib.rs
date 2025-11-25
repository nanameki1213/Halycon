#![no_std]

use mmio_core::MmioHandler;
use spin::Mutex

static VIRTIO_MMIO_REGISTER: Mutex<VirtioMmioRegister> = Mutex::new(VirtioMmioRegister::new());

pub struct Virtio;

impl MmioHandler for Virtio {
    fn read(&self, offset: usize) -> usize {
        let mut value = virtio_mmio.get_virtio_mmio(offset);

        match offset {
            VIRTIO_MMIO_VERSION => {
                virtio_mmio.set_virtio_mmio(VIRTIO_MMIO_DEVICE_FEATURES_SEL, 0);
                VIRTIO_MMIO_REGISTER.lock().device_features_low =
                    virtio_mmio.get_virtio_mmio(VIRTIO_MMIO_DEVICE_FEATURES);
                virtio_mmio.set_virtio_mmio(VIRTIO_MMIO_DEVICE_FEATURES_SEL, 1);
                VIRTIO_MMIO_REGISTER.lock().device_features_high =
                    virtio_mmio.get_virtio_mmio(VIRTIO_MMIO_DEVICE_FEATURES);
            }
            VIRTIO_MMIO_QUEUE_READY => {
                value = VIRTIO_DEFAULT_INDEX;
            }
            VIRTIO_MMIO_STATUS => {
                value = VIRTIO_MMIO_REGISTER.lock().status;
            }
            VIRTIO_MMIO_DEVICE_FEATURES => {
                if VIRTIO_MMIO_REGISTER.lock().device_features_sel == 0 {
                    value = VIRTIO_MMIO_REGISTER.lock().device_features_low;
                } else {
                    value = VIRTIO_MMIO_REGISTER.lock().device_features_high;
                }
            }
            _ => {}
        }

        // println!("read: {:#X}, {:#X}", offset, value);
        Ok(value)
        }
}

// Real Device
// TODO: Decouple host device handling from this code
// by introducing a host device package/abstraction.


