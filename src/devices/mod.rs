pub mod display;
pub mod keyboard;
pub mod terminal;

pub trait Device {
    fn read(&self, offset: usize) -> u8;
    fn write(&mut self, offset: usize, value: u8);
}

// Forwards the device interface through boxed trait objects, letting
// collections of heterogeneous devices (e.g. in a builder) be registered
// without unwrapping.
impl Device for Box<dyn Device> {
    fn read(&self, offset: usize) -> u8 {
        self.as_ref().read(offset)
    }

    fn write(&mut self, offset: usize, value: u8) {
        self.as_mut().write(offset, value);
    }
}
