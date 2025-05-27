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
            if let Some(v) = self.read(from + i as u16) {
                if self.write(to + i as u16, v) {
                    continue;
                }
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
        for (idx, inst) in program.iter().enumerate() {
            if !self.write2((idx * 2) as u16, *inst) {
                return false;
            }
        }
        
        self.as_read_only(0_u16, program.len() as u16)
    }

    pub fn as_read_only(&mut self, addr: u16, len: u16) -> bool {
        if self.read_only.is_empty() {
            self.read_only.push((addr, len));
            return true;
        }

        for (in_addr, in_len) in self.read_only.iter() {
            // check if it overlaps
            let overlaps = addr >= *in_addr && addr < (*in_addr + *in_len) ||
                *in_addr >= addr && *in_addr < (addr + len);
            if overlaps {
                return false;
            }
        }   

        self.read_only.push((addr, len));
        true
    }

    fn is_read_only(&self, addr: u16) -> bool {
        for (start_addr, len) in self.read_only.iter() {
            // the read-only section contains the given address
            if addr >= *start_addr && addr < (*start_addr + len) {
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
        } else {
            return None;
        }
    }

    fn write(&mut self, addr: u16, value: u8) -> bool {
        if self.is_read_only(addr) {
            return false; 
        }

        if (addr as usize) < self.size {
            self.bytes[addr as usize] = value;
            return true;
        } else {
            false
        }
    }
}
