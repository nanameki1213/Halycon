#![no_std]

use core::cell::UnsafeCell;
use core::cmp::min;
use core::sync::atomic::{AtomicU32, Ordering};

const PAGE_SIZE: usize = 4096;
const HEADER_SIZE: usize = 8; // head(u32) + tail(u32)
const BUF_LEN: usize = PAGE_SIZE - HEADER_SIZE; // 4088 bytes

/// Shared-memory ring buffer (SPSC).
///
/// # Concurrency contract (IMPORTANT)
/// - `push()` must be called only by the *single producer* (e.g. hypervisor).
/// - `pop()`  must be called only by the *single consumer* (e.g. VM).
/// - If you need MPSC/MPMC, you need a different design (locks or CAS loops).
#[repr(C)]
pub struct ShmRing {
    head: AtomicU32, // consumer advances head
    tail: AtomicU32, // producer advances tail
    buf: UnsafeCell<[u8; BUF_LEN]>,
}

// We provide Sync because interior mutation is controlled by SPSC + atomics ordering.
unsafe impl Sync for ShmRing {}

impl ShmRing {
    pub const fn new() -> Self {
        Self {
            head: AtomicU32::new(0),
            tail: AtomicU32::new(0),
            buf: UnsafeCell::new([0u8; BUF_LEN]),
        }
    }

    #[inline]
    pub const fn capacity(&self) -> u32 {
        BUF_LEN as u32
    }

    /// How many bytes are currently stored (readable).
    #[inline]
    pub fn readable_len(&self) -> u32 {
        let cap = self.capacity();
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        if tail >= head {
            tail - head
        } else {
            cap - (head - tail)
        }
    }

    /// How many bytes can be written right now.
    /// We keep 1 byte empty to distinguish full vs empty.
    #[inline]
    pub fn writable_len(&self) -> u32 {
        let cap = self.capacity();
        let used = self.readable_len();
        cap - used - 1
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.readable_len() == 0
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.writable_len() == 0
    }

    /// Producer: push bytes into the ring.
    /// Returns how many bytes were actually written (0..=data.len()).
    pub fn push(&self, data: &[u8]) -> usize {
        let cap = self.capacity() as usize;
        if cap == 0 || data.is_empty() {
            return 0;
        }

        // Producer reads head (written by consumer). Acquire pairs with consumer's head.store(Release).
        let head = self.head.load(Ordering::Acquire) as usize;
        // Producer's own tail can be Relaxed (only producer writes it).
        let tail = self.tail.load(Ordering::Relaxed) as usize;

        // used = distance(head -> tail) in ring space
        let used = if tail >= head {
            tail - head
        } else {
            cap - (head - tail)
        };

        // Keep one byte empty.
        let free = cap - used - 1;
        if free == 0 {
            return 0;
        }

        let n = min(data.len(), free);

        // Two-chunk copy to handle wrap-around.
        let first = min(n, cap - tail);
        let second = n - first;

        unsafe {
            let buf = &mut *self.buf.get();
            buf[tail..tail + first].copy_from_slice(&data[..first]);
            if second != 0 {
                buf[..second].copy_from_slice(&data[first..first + second]);
            }
        }

        // Publish the written data by updating tail with Release.
        let new_tail = (tail + n) % cap;
        self.tail.store(new_tail as u32, Ordering::Release);
        n
    }

    /// Consumer: pop bytes from the ring into `out`.
    /// Returns how many bytes were actually read (0..=out.len()).
    pub fn pop(&self, out: &mut [u8]) -> usize {
        let cap = self.capacity() as usize;
        if cap == 0 || out.is_empty() {
            return 0;
        }

        // Consumer reads tail (written by producer). Acquire pairs with producer's tail.store(Release).
        let tail = self.tail.load(Ordering::Acquire) as usize;
        // Consumer's own head can be Relaxed (only consumer writes it).
        let head = self.head.load(Ordering::Relaxed) as usize;

        let used = if tail >= head {
            tail - head
        } else {
            cap - (head - tail)
        };

        if used == 0 {
            return 0;
        }

        let n = min(out.len(), used);

        // Two-chunk copy to handle wrap-around.
        let first = min(n, cap - head);
        let second = n - first;

        unsafe {
            let buf = &*self.buf.get();
            out[..first].copy_from_slice(&buf[head..head + first]);
            if second != 0 {
                out[first..first + second].copy_from_slice(&buf[..second]);
            }
        }

        // Publish that we've consumed data by updating head with Release.
        let new_head = (head + n) % cap;
        self.head.store(new_head as u32, Ordering::Release);
        n
    }

    /// Optional: reset (only safe when no concurrent access).
    pub fn clear(&self) {
        self.head.store(0, Ordering::Relaxed);
        self.tail.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn size_is_one_page() {
        assert_eq!(core::mem::size_of::<ShmRing>(), 4096);
        assert_eq!(BUF_LEN, 4096 - 8);
    }

    #[test]
    fn basic_push_pop() {
        let r = ShmRing::new();
        let msg = b"hello";
        assert_eq!(r.push(msg), 5);

        let mut out = [0u8; 5];
        assert_eq!(r.pop(&mut out), 5);
        assert_eq!(&out, msg);

        assert!(r.is_empty());
        assert_eq!(r.pop(&mut out), 0);
    }

    #[test]
    fn wrap_around_preserves_order() {
        let r = ShmRing::new();
        let cap = r.capacity() as usize;

        // Fill close to the end to force wrap on next pushes.
        let a_len = cap - 20; // keep some room for operations
        let a: Vec<u8> = (0..a_len).map(|i| (i % 251) as u8).collect();
        assert_eq!(r.push(&a), a_len);

        // Pop some bytes to move head forward.
        let pop1 = 200;
        let mut tmp = vec![0u8; pop1];
        assert_eq!(r.pop(&mut tmp), pop1);
        assert_eq!(&tmp[..], &a[..pop1]);

        // Now push more to wrap around.
        let b_len = 150;
        let b: Vec<u8> = (0..b_len).map(|i| (200 + i) as u8).collect();
        assert_eq!(r.push(&b), b_len);

        // Pop remaining from a plus all b, and verify order.
        let mut out = vec![0u8; (a_len - pop1) + b_len];
        assert_eq!(r.pop(&mut out), out.len());

        assert_eq!(&out[..a_len - pop1], &a[pop1..]);
        assert_eq!(&out[a_len - pop1..], &b[..]);
    }

    #[test]
    fn full_and_partial_write() {
        let r = ShmRing::new();
        let cap = r.capacity() as usize;

        // Max storable is cap-1 (one byte kept empty).
        let fill = vec![0xAAu8; cap - 1];
        assert_eq!(r.push(&fill), cap - 1);
        assert!(r.is_full());

        // Further push should write 0.
        assert_eq!(r.push(&[1, 2, 3]), 0);

        // Pop some, then push again.
        let mut out = vec![0u8; 10];
        assert_eq!(r.pop(&mut out), 10);
        assert_eq!(out, vec![0xAAu8; 10]);

        let more = vec![0xBBu8; 20];
        assert_eq!(r.push(&more), 10);

        // Drain and check ordering: remaining AA then BB.
        let mut drain = vec![0u8; cap - 1];
        assert_eq!(r.pop(&mut drain), drain.len());

        assert_eq!(
            &drain[..(cap - 1) - 10],
            vec![0xAAu8; (cap - 1) - 10].as_slice()
        );
        assert_eq!(&drain[(cap - 1) - 10..], vec![0xBBu8; 10].as_slice());
    }

    #[test]
    fn concurrent_spsc_stress() {
        const N: usize = 50_000;

        let ring = Arc::new(ShmRing::new());

        let prod = {
            let ring = Arc::clone(&ring);
            thread::spawn(move || {
                let mut sent = 0usize;
                while sent < N {
                    let b = [(sent & 0xFF) as u8];
                    if ring.push(&b) == 1 {
                        sent += 1;
                    } else {
                        thread::yield_now();
                    }
                }
            })
        };

        let cons = {
            let ring = Arc::clone(&ring);
            thread::spawn(move || {
                let mut recv = 0usize;
                let mut out = [0u8; 1];
                while recv < N {
                    if ring.pop(&mut out) == 1 {
                        assert_eq!(out[0], (recv & 0xFF) as u8);
                        recv += 1;
                    } else {
                        thread::yield_now();
                    }
                }
            })
        };

        prod.join().unwrap();
        cons.join().unwrap();
        assert!(ring.is_empty());
    }
}
