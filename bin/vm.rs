use rust16vm::{
    devices::{keyboard::Keyboard, terminal::Terminal256},
    machine::{Machine, Register},
    memory::{self, Addressable, LinearMemory},
    mmio::MemoryWithDevices,
    rv16asm,
};

pub fn main() -> () {
    let program = rv16asm! {
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
        "STR A, B",             // flushes the stdout

        // now waits user input
        "MOV A, #0",
        "SUB SP, 2",            // place a flag into stack
                                // to define the end of the user input
        "STR SP, A",

        "MOV B, #0x0F",
        "MSL B, [#1 #4]",
        "MSL B, [#1 #6]",
        "MSL B, [#0 #2]",

        "LDR A, B",             // loads keyboard buffer into stack
        "SUB SP, 2",
        "STR SP, A",

        "MOV C, #13",           // only stops if the user hits enter
        "NEQ A, C",
        "CJP #60",

        "MOV B, #0x0F",         // print user input
        "MSL B, [#0 #7]",
        "MSL B, [#0 #5]",       
        
        "STR SP, C",
        "ADD SP, #2",
        "EQ C, #0",
        "CJP #90",

        "STR C, B",            // zero-terminated string
        "ADD B, #1",
        "CJP #72",

        "MOV A, #2",
        "MOV B, #0x0F",
        "MSL B, [#1 #4]",
        "MSL B, [#1 #7]",
        "MSL B, [#0 #1]",
        "STR A, B",             // flushes the stdout

        "ADD FLAGS, #1",        // halt machine
    };

    let mut memory = LinearMemory::new(63 * 1024); //63Kb

    assert!(memory.write_program(&program));

    let terminal = Terminal256::new();
    let keyboard = Keyboard::new();

    let mut memory = MemoryWithDevices::new(memory);

    memory.register_device(terminal, 0xF000, 259).unwrap();
    memory.register_device(keyboard, 0xF104, 2).unwrap();

    let mut machine = Machine::new(memory);
    // define the stack pointer to the memory end;
    machine.set_register(Register::SP, 0xFFFF);

    while let Ok(_) = machine.step() {}
}
