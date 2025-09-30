use std::env::{self};
use std::fs::File;
use std::io::{BufReader, Read, prelude::*, stdout};
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

    let mut machine = Machine::new_debug(memory, is_debug);
    // define the stack pointer to the memory end;
    machine.set_register(Register::SP, 0xFFFF);

    let mut stdout = stdout();
    let mut hit_dbg = false;
    
    loop {
        if is_debug {
            if hit_dbg {
                hit_dbg = false;
                match Terminal256::read_from_stdin().unwrap() {
                    TerminalAction::KeyPressed(c) if c == 'q' => break,
                    TerminalAction::KeyPressed(c) if c == 's' => {
                        hit_dbg = true;
                    },
                    TerminalAction::KeyPressed(c) if c == 'r' => {
                        print!("reading from stack:\r\n");

                        let mut text_mem_address = String::new();
                        loop {
                            let action = Terminal256::read_from_stdin();
                            match action  {
                                Ok(TerminalAction::NumberPressed(n)) => {
                                    print!("{}", n);
                                    stdout.flush().unwrap();
                                    text_mem_address.push(n);
                                }
                                Ok(TerminalAction::KeyPressedBackspace) => {
                                    // Erase the last character
                                    print!("\u{8} \u{8}"); // Move cursor back, print space, move cursor back again
                                    stdout.flush().unwrap(); // Ensure the changes are written
                                    text_mem_address.remove(text_mem_address.len()-1);
                                }
                                Ok(TerminalAction::KeyPressedEnter) => break,
                                _ => {}
                            }
                        }
                        print!("\r\n");

                        let mut text_size = String::new();
                        loop {
                            let action = Terminal256::read_from_stdin();
                            match action  {
                                Ok(TerminalAction::NumberPressed(n)) => {
                                    print!("{}", n);
                                    stdout.flush().unwrap();
                                    text_size.push(n);
                                }
                                Ok(TerminalAction::KeyPressedBackspace) => {
                                    // Erase the last character
                                    print!("\u{8} \u{8}"); // Move cursor back, print space, move cursor back again
                                    stdout.flush().unwrap(); // Ensure the changes are written
                                    text_size.remove(text_size.len()-1);
                                }
                                Ok(TerminalAction::KeyPressedEnter) => break,
                                _ => {}
                            }
                        }
                        print!("\r\n");

                        let mem_addr: u16 = text_mem_address.parse().unwrap();
                        let size: u16 = text_size.parse().unwrap();
                        let output = machine.read_from_memory(mem_addr, size);

                        for (idx, value) in output.iter().enumerate() {
                            print!(
                                "{}:\t{:#010b} | {:#04x} | {}\r\n",
                                mem_addr + (idx as u16),
                                value,
                                value,
                                value
                            );
                        }
                        stdout.flush().unwrap(); // Ensure the changes are written
                        hit_dbg = true;
                        continue;
                    },
                    TerminalAction::KeyPressedEnter => {}
                    _ => continue
                }
            }            
        }

        let r = machine.step();
        match r {
            Ok(State::Continue) => continue,
            Ok(State::Stop) => break,
            Ok(State::Debug) => {
                hit_dbg = true;
                continue;
            }
            Err(err) => {
                print!("error: {:?}\r\n", err);
                _ = stdout.flush().unwrap();
                break;
            }
        }
    }

    
}
