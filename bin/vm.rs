use std::env::{self};
use std::fs::File;
use std::io::{BufReader, Read, prelude::*};
use std::path::Path;

use crossterm::terminal as crossterm_terminal;
use rust16vm::devices::screen::ScreenOptions;
use rust16vm::devices::terminal::TerminalAction;
use rust16vm::machine::State;
use rust16vm::{
    devices::{keyboard::Keyboard, screen::ScreenDevice, terminal::Terminal256},
    machine::{Machine, Register},
    memory::{self, Addressable, LinearMemory},
    mmio::MemoryWithDevices,
    rv16asm,
};

pub fn main() -> () {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("expected 1 positional arg, received {}", args.len() - 1);
        return;
    }

    let is_debug = if let Some(last_arg) = args.last() {
        last_arg.eq(&String::from("--debug"))
    } else {
        false
    };

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

    loop {
        if is_debug {
            match Terminal256::read_from_stdin().unwrap() {
                TerminalAction::KeyPressed(c) if c == 'q' => break,
                TerminalAction::KeyPressedEnter => {}
                _ => continue
            }
        }

        let r = machine.step();
        match r {
            Ok(State::Continue) => continue,
            Ok(State::Stop) => break,
            Err(err) => {
                print!("error: {:?}\r\n", err);
                break;
            }
        }
    }
}
