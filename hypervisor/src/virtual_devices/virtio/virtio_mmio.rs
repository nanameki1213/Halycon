use core::fmt::Debug;

use crate::paging::resolve_address_stage2;
use mmio_core::MmioHandler;
use virtio::*;

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
        let mut value = 0;
        let register = self.device.mmio_state();

        match offset {
            VIRTIO_MMIO_VERSION => {
                value = VIRTIO_VERSION;
            }
            VIRTIO_MMIO_DEVICEID => {
                value = self.device.device_id() as usize;
            }
            VIRTIO_MMIO_QUEUE_READY => {
                value = VIRTIO_DEFAULT_INDEX as usize;
            }
            VIRTIO_MMIO_STATUS => {
                value = register.status as usize;
            }
            VIRTIO_MMIO_DEVICE_FEATURES => {
                if register.device_features_sel == 0 {
                    value = register.device_features_low as usize;
                } else {
                    value = register.device_features_high as usize;
                }
            }
            _ => {}
        }

        // println!("read: {:#X}, {:#X}", offset, value);
        value as usize
    }

    fn write(&mut self, offset: usize, value: usize) {
        // println!("write: {:#X}, {:#X}", offset, value as u32);

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
                used_ring.idx += 1;
            },
            VIRTIO_MMIO_QUEUE_READY => {}
            VIRTIO_MMIO_QUEUE_NUM => {
                self.device.mmio_state_mut().queue_num = value as u32;
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
