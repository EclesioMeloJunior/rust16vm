pub mod keyboard;
pub mod terminal;

pub trait Device {
    fn read(&self, offset: usize) -> u8;
    fn write(&mut self, offset: usize, value: u8);
}
