use rust16vm::{
    devices::terminal::Terminal256, machine::Machine, memory::{self, Addressable, LinearMemory}, mmio::MemoryWithDevices, rv16asm
};


pub fn main() -> () {
    let program = rv16asm!{ 
        "MOV A, #72",           // 'h'
        "MOV B, #0x0F",
        "MSL B, [#0 #7]",
        "MSL B, [#0 #5]",      // terminal buffer start
        "STR A, B",

        "MOV A, #101",           // 'e'
        "ADD B, #1",
        "STR A, B",

        "MOV A, #108",           // 'l'
        "ADD B, #1",
        "STR A, B",

        "MOV A, #108",           // 'l'
        "ADD B, #1",
        "STR A, B",

        "MOV A, #111",           // 'o'
        "ADD B, #1",
        "STR A, B",

        "MOV A, #2",
        "MOV B, #0x0F",
        "MSL B, [#1 #4]",
        "MSL B, [#1 #7]",
        "MSL B, [#0 #1]",
        "STR A, B",

        "ADD FLAGS, #1",        // halt machine
    };

    let mut memory = LinearMemory::new(64 * 1024); //64Kb

    assert!(memory.write_program(&program));

    let terminal = Terminal256::new();

    let mut memory = MemoryWithDevices::new(memory);

    memory.register_device(terminal, 0xF000, 512).unwrap();

    let mut machine = Machine::new(memory);

    while let Ok(_) = machine.step() {}
}
