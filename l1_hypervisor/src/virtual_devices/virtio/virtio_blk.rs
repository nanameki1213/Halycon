use block::{BlockDevice, virtio_blk::VirtioBlkReq};
use virtio::VRingDesc;
use virtio_core::{VIRTQ_ENTRY_NUM, virtio_blk::VirtioBlkConfig};

use crate::paging::resolve_address_stage2;

use super::virtio_mmio::{VirtioDevice, VirtioMmioRegister};

#[derive(Debug)]
pub struct VirtioBlkDevice<D: BlockDevice> {
    block_device: D,
    mmio_state: VirtioMmioRegister,
    config: VirtioBlkConfig,
}

impl<D: BlockDevice> VirtioBlkDevice<D> {
    pub fn new(block_device: D) -> Self {
        let capacity = block_device.get_capacity();

        let mut config = VirtioBlkConfig::new();
        config.capacity = capacity as u64;

        VirtioBlkDevice {
            block_device,
            mmio_state: VirtioMmioRegister::new(),
            config,
        }
    }
}

impl<T: BlockDevice> VirtioDevice for VirtioBlkDevice<T> {
    fn device_id(&self) -> u32 {
        2
    }

    fn notify(&mut self, desc_ring: &[VRingDesc; VIRTQ_ENTRY_NUM as usize]) {
        let request_address = resolve_address_stage2(desc_ring[0].addr as usize).unwrap();
        let data_address = resolve_address_stage2(desc_ring[1].addr as usize).unwrap();
        let status_address = resolve_address_stage2(desc_ring[2].addr as usize).unwrap() as *mut u8;
        let virtio_blk_req = unsafe { &mut *(request_address as *mut VirtioBlkReq) };

        if desc_ring[1].flags & VRingDesc::VIRTQ_DESC_F_WRITE as u16 != 0 {
            let count = desc_ring[1].len as usize / block::SECTOR_SIZE;
            let _ = self.block_device.read_write_disk(
                data_address as *mut usize,
                virtio_blk_req.sector,
                count,
                false,
            );
        }

        unsafe {
            core::ptr::write_volatile(status_address, block::virtio_blk::VIRTIO_BLK_S_OK as u8);
        }
    }

    fn mmio_state(&self) -> &VirtioMmioRegister {
        &self.mmio_state
    }

    fn mmio_state_mut(&mut self) -> &mut VirtioMmioRegister {
        &mut self.mmio_state
    }

    fn read_config(&self, offset: usize) -> u32 {
        self.config.read_as_bytes(offset)
    }
}
