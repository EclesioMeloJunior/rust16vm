use rust16vm::{
    machine::Machine,
    memory::{Addressable, LinearMemory},
};

pub fn main() -> () {
    let mut mem = LinearMemory::new(8 * 1024); //8Kb
    // 3 instructions to fill a register with ones
    mem.write2(0, 0b1111111110000001); // MOV A, #8
    mem.write2(2, 0b1111111010000010); // MSL A, 5 #31
    mem.write2(4, 0b0001110100000010); // MSL A, 2 #3

    let mut vm = Machine::new(mem);

    let _ = vm.step().unwrap();
    vm.print_regs();
    let _ = vm.step().unwrap();
    vm.print_regs();
    let _ = vm.step().unwrap();
    vm.print_regs();
}
