use std::env::{self};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use rust16vm::devices::screen::ScreenOptions;
use rust16vm::{
    devices::{keyboard::Keyboard, screen::ScreenDevice, terminal::Terminal256},
    machine::{Machine, Register},
    memory::{self, Addressable, LinearMemory},
    mmio::MemoryWithDevices,
    rv16asm,
};
/*
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
        "SUB SP, #2",            // place a flag into stack
                                // to define the end of the user input
        "STR A, SP",

        "MOV B, #0x0F",
        "MSL B, [#1 #4]",
        "MSL B, [#1 #6]",
        "MSL B, [#0 #2]",       // B has add 0xF104 that reads the keyboard state

        "LDR A, B",             // loads keyboard status into stack
        "EQ A, #0",             // keyboard buffer is empty, wait input
        "CJP #60",

        "SUB SP, #2",
        "MOV B, #0x0F",
        "MSL B, [#1 #4]",
        "MSL B, [#1 #6]",
        "MSL B, [#1 #2]",       // B has addr 0xF105 that reads the keyboard buffer

        "LDR A, B",
        "STR A, SP",

        "MOV C, #13",           // only stops if the user hits enter
        "NEQ A, C",
        "CJP #52",

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
*/

pub fn main() -> () {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("expected 1 positional arg, received {}", args.len() - 1);
        return;
    }

    let path = Path::new(&args[1]);
    let open_file = File::open(path);

    let mut input_program: Vec<u8> = Vec::new();
    match open_file {
        Ok(file) => {
            let mut buf = BufReader::new(file);
            match buf.read_to_end(&mut input_program) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("reading input program: {}", err);
                    return;
                }
            }
        }
        Err(err) => {
            eprintln!("opening input program: {}", err);
            return;
        }
    };

    let program: Vec<u16> = input_program
        .chunks(2)
        .map(|chunk| (chunk[1] as u16) << 8 | (chunk[0] as u16))
        .collect();

    let mut memory = LinearMemory::new(1 << 16); //63Kb

    assert!(memory.write_program(&program));

    let mut opts = ScreenOptions::default();
    opts.debug_instructions(program.clone(), 0);

    let terminal = Terminal256::new();
    let mut memory = MemoryWithDevices::new(memory);

    memory.register_device(terminal, 0xF000, 259).unwrap();

    let mut machine = Machine::new(memory);
    // define the stack pointer to the memory end;
    machine.set_register(Register::SP, 0xFFFF);

    while let Ok(_) = machine.step() {}
}
