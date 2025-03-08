use crate::{devices::Device, memory::{Addressable, LinearMemory}};

pub struct DeviceBus {
    // maps memory regions to devices
    // (start addr, end addr, device index)
    mmap: Vec<(u16, u16, usize)>,
    devices: Vec<Box<dyn Device>>,
}

impl DeviceBus {
    pub fn new() -> Self {
        Self {
            mmap: Vec::new(),
            devices: Vec::new(),
        }
    }

    pub fn register_device<D: Device + 'static>(
        &mut self,
        device: D,
        start_addr: u16,
        size: u16,
    ) -> Result<(), String> {
        let end_addr = start_addr
            .checked_add(size)
            .ok_or_else(|| "Memory region overflow".to_string())?;

        for (existing_start, existing_end, _) in &self.mmap {
            if start_addr <= *existing_end && end_addr >= *existing_start {
                return Err("Memory region overlaps with existing device".to_string());
            }
        }

        let dev_idx = self.devices.len();
        self.devices.push(Box::new(device));

        self.mmap.push((start_addr, end_addr, dev_idx));

        Ok(())
    }

    fn find_service(&self, address: u16) -> Option<(usize, u16)> {
        for (start, end, dv_idx) in &self.mmap {
            if address >= *start && address < *end {
                return Some((*dv_idx, address-*start))
            }
        }

        None
    }

    pub fn read(&self, address: u16) -> Option<u8> {
        if let Some((dv_idx, offset)) = self.find_service(address) {
            return Some(self.devices[dv_idx].read(offset as usize))
        }

        None
    }

    pub fn write(&mut self, address: u16, value: u8) -> bool {
        if let Some((dv_idx, offset)) = self.find_service(address) {
            self.devices[dv_idx].write(offset as usize, value);
            return true
        }

        false
    }
}

pub struct MemoryWithDevices {
    linear_memory: LinearMemory,
    device_bus: DeviceBus,
}

impl MemoryWithDevices {
    pub fn new(memory: LinearMemory) -> Self {
        Self {
            linear_memory: memory,
            device_bus: DeviceBus::new()
        }
    }

    pub fn register_device<D: Device + 'static>(
        &mut self,
        device: D,
        start_addr: u16,
        size: u16,
    ) -> Result<(), String> {
        self.device_bus.register_device(device, start_addr, size)
    }
}

impl Addressable for MemoryWithDevices {
    fn read(&self, addr: u16) -> Option<u8> {
        if let Some(value) = self.device_bus.read(addr) {
            Some(value)
        } else {
            self.linear_memory.read(addr)
        }
    }

    fn write(&mut self, addr: u16, value: u8) -> bool {
        if self.device_bus.write(addr, value) {
            true
        } else {
            self.linear_memory.write(addr, value)
        }
    }
}
