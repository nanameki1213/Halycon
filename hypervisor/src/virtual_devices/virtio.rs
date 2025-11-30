use crate::PASS_THROUGH_VIRTIO_BLK_DEVICE;
use crate::PASS_THROUGH_VIRTIO_MMIO;
use crate::paging::resolve_address_stage2;
use block::BlockDevice;
use block::virtio_blk;
use mmio_core::MmioHandler;
use spin::Mutex;
use virtio::*;

static VIRTIO_MMIO_REGISTER: Mutex<VirtioMmioRegister> = Mutex::new(VirtioMmioRegister::new());

#[derive(Debug)]
pub struct Virtio;

impl MmioHandler for Virtio {
    fn read(&self, offset: usize) -> usize {
        let virtio_mmio: &VirtioMmio;
        match PASS_THROUGH_VIRTIO_MMIO.get() {
            Some(mmio) => virtio_mmio = mmio,
            None => panic!(),
        }

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
        value as usize
    }

    fn write(&self, offset: usize, value: usize) {
        // println!("write: {:#X}, {:#X}", offset, value as u32);

        match offset {
            VIRTIO_MMIO_QUEUE_NOTIFY => unsafe {
                if value as u32 != VIRTIO_MMIO_REGISTER.lock().queue_sel {
                    // TODO: Set STATUS Register
                    return;
                }

                let desc_address =
                    resolve_address_stage2(VIRTIO_MMIO_REGISTER.lock().desc_address as usize)
                        .unwrap();
                let desc_ring = &mut *core::ptr::slice_from_raw_parts_mut(
                    desc_address as *mut VRingDesc,
                    VIRTQ_ENTRY_NUM as usize,
                );
                let request_address = resolve_address_stage2(desc_ring[0].addr as usize).unwrap();
                let data_address = resolve_address_stage2(desc_ring[1].addr as usize).unwrap();
                let status_address =
                    resolve_address_stage2(desc_ring[2].addr as usize).unwrap() as *mut u8;
                let virtio_blk_req = &mut *(request_address as *mut virtio_blk::VirtioBlkReq);

                // println!("\n");
                // for i in 0..3 {
                //     println!("desc[{}]: addr: {:#x}, len: {}", i, desc_ring[i].addr as usize, desc_ring[i].len);
                // }

                // println!("virtio: {}", virtio_blk_req.sector);
                let mutex = PASS_THROUGH_VIRTIO_BLK_DEVICE.get_unchecked();
                let mut block_device = mutex.lock();

                if desc_ring[1].flags & VRingDesc::VIRTQ_DESC_F_WRITE as u16 != 0 {
                    let count = desc_ring[1].len as usize / block::SECTOR_SIZE;
                    let _ = block_device.read_write_disk(
                        data_address as *mut usize,
                        virtio_blk_req.sector,
                        count,
                        false,
                    );
                }

                let device_address =
                    resolve_address_stage2(VIRTIO_MMIO_REGISTER.lock().device_address as usize)
                        .unwrap();
                let used_ring = &mut *(device_address as *mut VRingUsed);
                used_ring.idx += 1;

                core::ptr::write_volatile(status_address, virtio_blk::VIRTIO_BLK_S_OK as u8);
            },
            VIRTIO_MMIO_QUEUE_READY => {}
            VIRTIO_MMIO_QUEUE_NUM => {
                VIRTIO_MMIO_REGISTER.lock().queue_num = value as u32;
            }
            VIRTIO_MMIO_QUEUE_SEL => {
                VIRTIO_MMIO_REGISTER.lock().queue_sel = value as u32;
            }
            VIRTIO_MMIO_DEVICE_FEATURES_SEL => {
                VIRTIO_MMIO_REGISTER.lock().device_features_sel = value as u32;
            }
            VIRTIO_MMIO_DRIVER_FEATURES_SEL => {
                VIRTIO_MMIO_REGISTER.lock().driver_features_sel = value as u32;
            }
            VIRTIO_MMIO_DRIVER_FEATURES => {
                if VIRTIO_MMIO_REGISTER.lock().driver_features_sel == 0 {
                    VIRTIO_MMIO_REGISTER.lock().driver_features_low = value as u32;
                } else {
                    VIRTIO_MMIO_REGISTER.lock().driver_features_high = value as u32;
                }
            }
            VIRTIO_MMIO_STATUS_FEATURES_OK => {
                VIRTIO_MMIO_REGISTER.lock().status |= VIRTIO_MMIO_STATUS_FEATURES_OK as u32;
            }
            VIRTIO_MMIO_STATUS => {
                VIRTIO_MMIO_REGISTER.lock().status = value as u32;
            }
            VIRTIO_MMIO_DESC_LOW => {
                VIRTIO_MMIO_REGISTER.lock().desc_address = value as u32 as u64;
            }
            VIRTIO_MMIO_DRIVER_LOW => {
                VIRTIO_MMIO_REGISTER.lock().driver_address = value as u32 as u64;
            }
            VIRTIO_MMIO_DEVICE_LOW => {
                VIRTIO_MMIO_REGISTER.lock().device_address = value as u32 as u64;
            }
            _ => {}
        }
    }
}
