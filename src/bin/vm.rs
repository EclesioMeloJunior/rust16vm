use rust16vm::{memory::LinearMemory, machine::Machine};

pub fn main() -> () {
    let mem = LinearMemory::new(8 * 1024); //8Kb
    let mut vm = Machine::new(mem);

    let _ = vm.step().unwrap();
}
