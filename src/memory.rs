// Addressable is a trait that defines
// any implementation over a memory where
// the values can have an address.
// 
//     Address [u16]    Memory Content [u8]
//     +-----------+    +----------------+
//     | 0x0000    | -> |     0xA3       |  Single byte at address 0
//     +-----------+    +----------------+
//     | 0x0001    | -> |     0xF7       |  Single byte at address 1
//     +-----------+    +----------------+
//     | 0x0002    | -> |     0x21       |
//     +-----------+    +----------------+
//     | 0x0003    | -> |     0x5E       |
//     +-----------+    +----------------+
//             ...             ...
//     +-----------+    +----------------+
//     | 0xFFFE    | -> |     0x8B       |  Address 65534
//     +-----------+    +----------------+
//     | 0xFFFF    | -> |     0xC4       |  Address 65535 (max for u16)
//     +-----------+    +----------------+
//    
//         u16 value spanning two bytes:
//         
//     +-----------+    +----------------+
//     | 0x0004    | -> |     0x42       |  Together form
//     +-----------+    +----------------+  u16 value 0x4291
//     | 0x0005    | -> |     0x91       |  (16962 in decimal)
//     +-----------+    +----------------+
//    
//     Total addressable memory: 65536 bytes (64 "KiB")
pub trait Addressable {
    fn read(&self, addr: u16) -> Option<u8>;
    fn write(&mut self, addr: u16, value: u8) -> bool;

    fn read2(&self, addr: u16) -> Option<u16> {
        self.read(addr).and_then(|lo| {
            self.read(addr + 1)
                .map(|hi| (lo as u16) | ((hi as u16) << 8))
        })
    }

    fn write2(&mut self, addr: u16, value: u16) -> bool {
        let lo = (value & 0x00ff) as u8;
        let hi = ((value & 0xff00) >> 8) as u8;

        self.write(addr, lo) && self.write(addr + 1, hi)
    }

    /// copy places the values at [from ... from + n[
    /// at [to ... to + n[
    /// does not changes the values at `from` range
    fn copy(&mut self, from: u16, to: u16, n: usize) -> bool {
        for i in 0..n {
            if let Some(v) = self.read(from + i as u16)
                && self.write(to + i as u16, v) {
                    continue;
                }

            return false;
        }

        true
    }
}

pub struct LinearMemory {
    bytes: Vec<u8>,
    size: usize,
    // read_only holds non-overlapping read-only regions
    read_only: Vec<(u16,  u16)>,
}

impl LinearMemory {
    pub fn new(n: usize) -> Self {
        Self {
            bytes: vec![0; n],
            size: n,
            read_only: vec![],
        }
    }

    // write program stores the set of instructions and mark
    // the region as read-only
    pub fn write_program(&mut self, program: &[u16]) -> bool {
        self.write_program_at(0, program)
    }

    // write_program_at stores the set of instructions starting at `base`
    // and marks the region [base, base + 2 * len[ as read-only.
    // fails if the program crosses the end of the address space or
    // overlaps a read-only region
    pub fn write_program_at(&mut self, base: u16, program: &[u16]) -> bool {
        for (idx, inst) in program.iter().enumerate() {
            let Some(at) = base.checked_add((idx * 2) as u16) else {
                return false;
            };

            if !self.write2(at, *inst) {
                return false;
            }
        }

        // each instruction occupies 2 bytes; saturates for the theoretical
        // 64KiB program where the exclusive end does not fit in u16
        let len = (program.len() as u32 * 2).min(u16::MAX as u32) as u16;
        self.as_read_only(base, len)
    }

    pub fn as_read_only(&mut self, addr: u16, len: u16) -> bool {
        // u32 math: a region ending at 0x10000 overflows u16
        let new_start = addr as u32;
        let new_end = new_start + len as u32;

        if self.read_only.is_empty() {
            self.read_only.push((addr, len));
            return true;
        }

        for (in_addr, in_len) in &self.read_only {
            // check if it overlaps
            let in_start = *in_addr as u32;
            let in_end = in_start + *in_len as u32;
            if new_start < in_end && in_start < new_end {
                return false;
            }
        }

        self.read_only.push((addr, len));
        true
    }

    fn is_read_only(&self, addr: u16) -> bool {
        let at = addr as u32;
        for (start_addr, len) in &self.read_only {
            // the read-only section contains the given address
            let start = *start_addr as u32;
            if at >= start && at < start + *len as u32 {
                return true
            }
        }

        false
    }
}

impl Addressable for LinearMemory {
    fn read(&self, addr: u16) -> Option<u8> {
        if (addr as usize) < self.size {
            return Some(self.bytes[addr as usize]);
        }
        return None;
    }

    fn write(&mut self, addr: u16, value: u8) -> bool {
        if self.is_read_only(addr) {
            return false; 
        }

        if (addr as usize) < self.size {
            self.bytes[addr as usize] = value;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn write_program_at_loads_and_marks_read_only() {
        let mut mem = LinearMemory::new(1024);
        assert!(mem.write_program_at(0x0100, &[1, 2, 3]));

        // little-endian words land at base, base+2, base+4
        assert_eq!(mem.read2(0x0100), Some(1));
        assert_eq!(mem.read2(0x0102), Some(2));
        assert_eq!(mem.read2(0x0104), Some(3));

        // the loaded region is read-only, everything else is not
        assert!(!mem.write(0x0100, 42));
        assert!(!mem.write(0x0105, 42));
        assert!(mem.write(0x0106, 42));
        assert!(mem.write(0x00FF, 42));
    }

    #[test]
    fn program_crossing_the_address_space_end_fails() {
        let mut mem = LinearMemory::new(1 << 16);
        // 0x8000 words starting at 0xFF00 would end at 0x1FF00
        let program = vec![0u16; 0x8000];
        assert!(!mem.write_program_at(0xFF00, &program));
    }

    #[test]
    fn read_only_regions_handle_the_address_space_end() {
        let mut mem = LinearMemory::new(1 << 16);
        // a region ending exactly at 0x10000 must not overflow the u16 math
        assert!(mem.as_read_only(0xFFF0, 0x0010));

        assert!(!mem.write(0xFFFF, 1));
        assert!(!mem.write(0xFFF0, 1));
        assert!(mem.write(0xFFEF, 1));
    }
}
