use core::fmt::Debug;

use crate::paging::resolve_address_stage2;
use crate::println;
use mmio_core::MmioHandler;
use virtio::*;
use virtio_core::*;

pub trait VirtioDevice {
    fn device_id(&self) -> u32;

    fn notify(&mut self, desc_ring: &[VRingDesc; VIRTQ_ENTRY_NUM as usize]);

    fn mmio_state(&self) -> &VirtioMmioRegister;

    fn mmio_state_mut(&mut self) -> &mut VirtioMmioRegister;
}

#[derive(Debug)]
pub struct VirtioMmioTransport<D: VirtioDevice> {
    device: D,
}

impl<D: VirtioDevice> VirtioMmioTransport<D> {
    pub fn new(device: D) -> Self {
        VirtioMmioTransport { device }
    }
}

impl<D: VirtioDevice + Debug + Send> MmioHandler for VirtioMmioTransport<D> {
    fn read(&self, offset: usize) -> usize {
        let register = self.device.mmio_state();

        let value = match offset {
            VIRTIO_MMIO_MAGIC => VIRTIO_MMIO_MAGIC_VALUE,
            VIRTIO_MMIO_VERSION => VIRTIO_VERSION,
            VIRTIO_MMIO_DEVICEID => self.device.device_id() as usize,
            VIRTIO_MMIO_VENDERID => 0,
            VIRTIO_MMIO_DEVICE_FEATURES => {
                if register.device_features_sel == 0 {
                    register.device_features_low as usize
                } else {
                    register.device_features_high as usize
                }
            }
            VIRTIO_MMIO_QUEUE_SIZE_MAX => VIRTQ_ENTRY_NUM as usize,
            VIRTIO_MMIO_QUEUE_READY => register.queue_ready as usize,
            VIRTIO_MMIO_STATUS => register.status as usize,
            _ => 0,
        };

        println!("read: {:#X}, {:#X}", offset, value);
        value as usize
    }

    fn write(&mut self, offset: usize, value: usize) {
        println!("write: {:#X}, {:#X}", offset, value as u32);

        match offset {
            VIRTIO_MMIO_QUEUE_NOTIFY => unsafe {
                let device = &mut self.device;
                if value as u32 != device.mmio_state().queue_sel {
                    // TODO: Set STATUS Register
                    return;
                }

                let desc_address =
                    resolve_address_stage2(device.mmio_state().desc_address as usize).unwrap();
                let desc_ring_slice = &mut *core::ptr::slice_from_raw_parts_mut(
                    desc_address as *mut VRingDesc,
                    VIRTQ_ENTRY_NUM as usize,
                );
                let desc_ring: [VRingDesc; VIRTQ_ENTRY_NUM as usize] = desc_ring_slice
                    .try_into()
                    .expect("Failed to convert slice to array.");

                device.notify(&desc_ring);

                let device_address =
                    resolve_address_stage2(device.mmio_state().device_address as usize).unwrap();
                let used_ring = &mut *(device_address as *mut VRingUsed);
                used_ring.idx = used_ring.idx.wrapping_add(1);
            },
            VIRTIO_MMIO_QUEUE_READY => {}
            VIRTIO_MMIO_QUEUE_SIZE => {
                self.device.mmio_state_mut().queue_size = value as u32;
            }
            VIRTIO_MMIO_QUEUE_SEL => {
                self.device.mmio_state_mut().queue_sel = value as u32;
            }
            VIRTIO_MMIO_DEVICE_FEATURES_SEL => {
                self.device.mmio_state_mut().device_features_sel = value as u32;
            }
            VIRTIO_MMIO_DRIVER_FEATURES_SEL => {
                self.device.mmio_state_mut().driver_features_sel = value as u32;
            }
            VIRTIO_MMIO_DRIVER_FEATURES => {
                if self.device.mmio_state().driver_features_sel == 0 {
                    self.device.mmio_state_mut().driver_features_low = value as u32;
                } else {
                    self.device.mmio_state_mut().driver_features_high = value as u32;
                }
            }
            VIRTIO_MMIO_STATUS_FEATURES_OK => {
                self.device.mmio_state_mut().status |= VIRTIO_MMIO_STATUS_FEATURES_OK as u32;
            }
            VIRTIO_MMIO_STATUS => {
                self.device.mmio_state_mut().status = value as u32;
            }
            VIRTIO_MMIO_DESC_LOW => {
                self.device.mmio_state_mut().desc_address = value as u32 as u64;
            }
            VIRTIO_MMIO_DRIVER_LOW => {
                self.device.mmio_state_mut().driver_address = value as u32 as u64;
            }
            VIRTIO_MMIO_DEVICE_LOW => {
                self.device.mmio_state_mut().device_address = value as u32 as u64;
            }
            _ => {}
        }
    }
}
