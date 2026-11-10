// vm — terminal emulator + debugger for rust16vm.
//
// The machine is assembled through the local `builder` module, whose setters
// map 1:1 to the CLI options below, so the emulator can be customized without
// touching code:
//
//   vm output.bin --debug --memory 0x8000 --stack-top 0x7FFF --width 100
//
// Layout:
//   +------------------------+----------------------+
//   |                        |      Registers       |
//   |    Program Output      +----------------------+
//   |    (mapped terminal)   |       Memory         |
//   +------------------------+----------------------+
//   | status bar: run/pause/step, PC, controls        |
//   +-------------------------------------------------+
//
// Controls:
//   Space : run / pause (resumes to the next breakpoint in --debug)
//   S     : step one instruction (when paused)
//   Up/Down or k/j : scroll the memory view
//   R     : reset the machine (reload the program)
//   Q/Esc : quit
//
// With --debug every `DBG` instruction in the program acts as a breakpoint:
// the emulator pauses as soon as one executes.

// the builder is the configuration API for the emulator; some setters exist
// for programmatic use and are not exercised by the CLI itself
#[allow(dead_code)]
mod builder;
mod ui;

use std::time::Duration;

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

use rust16vm::{
    devices::terminal::DEFAULT_TERM_SIZE,
    machine::{Machine, State},
    mmio::MemoryWithDevices,
};

use builder::{BuiltMachine, DisplayHandle, MachineBuilder, Program};

/// How many instructions to execute per frame while running.
const STEPS_PER_FRAME: usize = 2_000;

#[derive(Parser)]
#[command(
    name = "vm",
    version,
    about = "rust16vm terminal emulator + debugger",
    after_help = "controls: [Space] run/pause  [S] step  [Up/Down] memory  [R] reset  [Q] quit"
)]
struct Cli {
    /// Path to the assembled program (the .bin produced by `asm`)
    program: String,

    /// Treat every DBG instruction as a breakpoint
    #[arg(short = 'd', long)]
    debug: bool,

    /// Linear memory size in bytes (accepts hex, e.g. 0x10000)
    #[arg(long, default_value_t = 64 * 1024, value_parser = parse_usize)]
    memory: usize,

    /// Memory address where the program is loaded; execution starts here
    #[arg(long, default_value_t = 0x0000, value_parser = parse_u16)]
    program_base: u16,

    /// Initial SP value; the stack grows down from this address
    #[arg(long, default_value_t = 0xFFFF, value_parser = parse_u16)]
    stack_top: u16,

    /// Base address where the terminal device is mapped
    #[arg(long, default_value_t = 0xF000, value_parser = parse_u16)]
    term_base: u16,

    /// Screen width in columns
    #[arg(long, default_value_t = 80, value_parser = parse_usize)]
    width: usize,

    /// Screen height in rows
    #[arg(long, default_value_t = 25, value_parser = parse_usize)]
    height: usize,
}

/// Parses decimal or 0x-prefixed hexadecimal integers.
fn parse_usize(s: &str) -> Result<usize, String> {
    let s = s.trim();
    match s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        Some(hex) => usize::from_str_radix(hex, 16).map_err(|e| e.to_string()),
        None => s.parse::<usize>().map_err(|e| e.to_string()),
    }
}

/// Parses decimal or 0x-prefixed hexadecimal u16 values.
fn parse_u16(s: &str) -> Result<u16, String> {
    let s = s.trim();
    match s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        Some(hex) => u16::from_str_radix(hex, 16).map_err(|e| e.to_string()),
        None => s.parse::<u16>().map_err(|e| e.to_string()),
    }
}

fn main() {
    let cli = Cli::parse();

    let program = match Program::from_file(&cli.program) {
        Ok(program) => program,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };

    if program.is_empty() {
        eprintln!("program is empty");
        return;
    }

    let mut app = match App::new(program, cli) {
        Ok(app) => app,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app);
    ratatui::restore();

    if let Err(err) = result {
        eprintln!("emulator error: {err}");
    }
}

/// Assembles the machine from the CLI configuration through the builder.
fn build_machine(program: &Program, cli: &Cli) -> Result<BuiltMachine, String> {
    MachineBuilder::new()
        .memory_size(cli.memory)
        .program_at(cli.program_base, program.clone())
        .display_at(cli.term_base, DEFAULT_TERM_SIZE, cli.width, cli.height)
        .stack_top(cli.stack_top)
        .debug(cli.debug)
        .build()
}

pub(crate) struct App {
    pub(crate) machine: Machine<MemoryWithDevices>,
    pub(crate) grid: DisplayHandle,
    pub(crate) program: Program,
    pub(crate) running: bool,
    pub(crate) halted: bool,
    pub(crate) quit: bool,
    pub(crate) mem_offset: u16,
    pub(crate) instr_count: u64,
    pub(crate) status: String,
    /// Original CLI configuration, reused on reset.
    cli: Cli,
}

impl App {
    fn new(program: Program, cli: Cli) -> Result<Self, String> {
        let built = build_machine(&program, &cli)?;
        Ok(Self {
            machine: built.machine,
            grid: built.display.expect("display device was configured"),
            program,
            running: false,
            halted: false,
            quit: false,
            mem_offset: 0,
            instr_count: 0,
            status: String::from("ready - press Space to run"),
            cli,
        })
    }

    fn reset(&mut self) {
        let built = build_machine(&self.program, &self.cli)
            .expect("config validated at startup must still build");
        self.machine = built.machine;
        self.grid = built.display.expect("display device was configured");
        self.running = false;
        self.halted = false;
        self.mem_offset = 0;
        self.instr_count = 0;
        self.status = String::from("reset");
    }

    /// Executes a single instruction, updating run/halt state.
    fn step_once(&mut self) {
        if self.halted {
            return;
        }

        match self.machine.step() {
            Ok(State::Continue) => {
                self.instr_count += 1;
            }
            Ok(State::Debug) => {
                // a `DBG` breakpoint executed: pause so the user can inspect
                // registers/memory or step instruction by instruction
                self.instr_count += 1;
                self.running = false;
                self.status = String::from("breakpoint hit - Space resumes, S steps");
            }
            Ok(State::Stop) => {
                self.halted = true;
                self.running = false;
                self.status = String::from("halted (FLAGS bit 0 set)");
            }
            Err(err) => {
                self.halted = true;
                self.running = false;
                self.status = format!("error: {err}");
            }
        }
    }
}

fn handle_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => app.quit = true,
        KeyCode::Char(' ') => {
            if !app.halted {
                app.running = !app.running;
                app.status = if app.running {
                    String::from("running")
                } else {
                    String::from("paused")
                };
            }
        }
        KeyCode::Char('s') => {
            if !app.halted {
                app.running = false;
                app.step_once();
                app.status = String::from("stepped");
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.mem_offset = app.mem_offset.saturating_sub(16);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.mem_offset = app.mem_offset.saturating_add(16);
        }
        KeyCode::Char('r') => app.reset(),
        _ => {}
    }
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> std::io::Result<()> {
    while !app.quit {
        terminal.draw(|frame| ui::draw(frame, app))?;

        let timeout = if app.running && !app.halted {
            Duration::from_millis(8)
        } else {
            Duration::from_millis(150)
        };

        if event::poll(timeout)?
            && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press {
                    handle_key(app, key.code);
                }

        if app.running && !app.halted {
            for _ in 0..STEPS_PER_FRAME {
                app.step_once();
                if !app.running || app.halted {
                    break;
                }
            }
        }
    }

    Ok(())
}
