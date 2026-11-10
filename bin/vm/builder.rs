//! App-level construction API for [`rust16vm::machine::Machine`] instances.
//!
//! This module lives in the `vm` binary (not the library) because it is
//! user-facing convenience rather than VM core: it accumulates memory
//! configuration, a program, memory-mapped devices and initial register
//! values, then performs the fixed construction sequence (linear memory ->
//! device bus -> machine -> register presets) inside a single fallible
//! [`MachineBuilder::build`]. The CLI flags of `vm` map 1:1 to builder
//! setters.
//!
//! ```ignore
//! let built = MachineBuilder::new()
//!     .program(Program::from_file("output.bin")?)
//!     .display()
//!     .stack_top(0xFFFF)
//!     .build()?;
//! ```

use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
    sync::{Arc, Mutex},
};

use rust16vm::{
    devices::{
        display::{DisplayDevice, TerminalGrid, DISPLAY_HEIGHT, DISPLAY_WIDTH},
        terminal::{Terminal256, DEFAULT_TERM_BASE, DEFAULT_TERM_SIZE},
        Device,
    },
    machine::{Machine, Register},
    memory::LinearMemory,
    mmio::MemoryWithDevices,
};

/// Handle to the shared in-memory screen created by [`MachineBuilder::display`].
pub type DisplayHandle = Arc<Mutex<TerminalGrid>>;

/// Largest memory size addressable with `u16` addresses (64 KiB);
/// also the default memory size.
const MAX_MEMORY: usize = 1 << 16;

/// A program to be loaded in the machine memory.
#[derive(Clone, Debug, Default)]
pub struct Program(Vec<u16>);

impl Program {
    /// Interprets `bytes` as little-endian 16-bit instructions, the same
    /// layout the `asm` binary writes to disk.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let instructions = bytes
            .chunks(2)
            .map(|chunk| {
                let lo = u16::from(chunk[0]);
                let hi = u16::from(chunk.get(1).copied().unwrap_or(0));
                (hi << 8) | lo
            })
            .collect();

        Self(instructions)
    }

    /// Loads a program from a binary file produced by the `asm` binary.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|e| format!("opening {}: {e}", path.display()))?;

        let mut bytes = Vec::new();
        BufReader::new(file)
            .read_to_end(&mut bytes)
            .map_err(|e| format!("reading {}: {e}", path.display()))?;

        Ok(Self::from_bytes(&bytes))
    }

    /// The assembled instructions, ready to be written into memory.
    pub fn instructions(&self) -> &[u16] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<u16>> for Program {
    fn from(instructions: Vec<u16>) -> Self {
        Self(instructions)
    }
}

/// Everything needed to draw the shared display screen.
struct DisplayConfig {
    base: u16,
    size: u16,
    width: usize,
    height: usize,
}

/// A machine plus the handles for the devices created during [`MachineBuilder::build`].
///
/// Devices live inside the machine's device bus, so any shared state they
/// expose (like the display grid) must be handed back to the caller here.
pub struct BuiltMachine {
    pub machine: Machine<MemoryWithDevices>,
    /// Present only when [`MachineBuilder::display`] was used.
    pub display: Option<DisplayHandle>,
}


impl From<BuiltMachine> for Machine<MemoryWithDevices> {
    fn from(val: BuiltMachine) -> Self {
        val.machine
    }
}

/// Fluent builder for [`Machine`] over [`MemoryWithDevices`].
///
/// Setters are order-independent; all validation happens once in [`build`](MachineBuilder::build),
/// which reports errors from device registration (overlap, region overflow)
/// and program loading instead of panicking.
pub struct MachineBuilder {
    memory_size: usize,
    program: Option<Program>,
    /// Address where the program is loaded; the PC starts here.
    program_base: u16,
    devices: Vec<(Box<dyn Device>, u16, u16)>,
    display: Option<DisplayConfig>,
    terminal: Option<(u16, u16)>,
    registers: Vec<(Register, u16)>,
    debug: bool,
}

impl Default for MachineBuilder {
    fn default() -> Self {
        Self {
            memory_size: MAX_MEMORY,
            program: None,
            program_base: 0x0000,
            devices: Vec::new(),
            display: None,
            terminal: None,
            registers: Vec::new(),
            debug: false,
        }
    }
}

impl MachineBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the linear memory size in bytes (defaults to 64 KiB).
    pub fn memory_size(mut self, size: usize) -> Self {
        self.memory_size = size;
        self
    }

    /// Sets the program to load at address `0x0000` (replaces any previous one).
    pub fn program<P: Into<Program>>(mut self, program: P) -> Self {
        self.program_base = 0x0000;
        self.program = Some(program.into());
        self
    }

    /// Sets the program to load at an arbitrary memory address. The PC starts
    /// at `base` unless explicitly overridden with [`Self::set_register`].
    pub fn program_at<P: Into<Program>>(mut self, base: u16, program: P) -> Self {
        self.program_base = base;
        self.program = Some(program.into());
        self
    }

    /// Convenience: builds the program from raw little-endian bytes.
    pub fn program_bytes(mut self, bytes: &[u8]) -> Self {
        self.program = Some(Program::from_bytes(bytes));
        self
    }

    /// Registers a memory-mapped device at `[base, base + size[`.
    ///
    /// Registration (and its overlap checks) is deferred to `build`.
    pub fn device<D: Device + 'static>(mut self, device: D, base: u16, size: u16) -> Self {
        self.devices.push((Box::new(device), base, size));
        self
    }

    /// Maps an in-memory display device (grid shared through `BuiltMachine::display`)
    /// at the conventional terminal address.
    pub fn display(self) -> Self {
        self.display_at(
            DEFAULT_TERM_BASE,
            DEFAULT_TERM_SIZE,
            DISPLAY_WIDTH,
            DISPLAY_HEIGHT,
        )
    }

    /// Maps an in-memory display device with explicit address and grid size.
    pub fn display_at(mut self, base: u16, size: u16, width: usize, height: usize) -> Self {
        self.display = Some(DisplayConfig {
            base,
            size,
            width,
            height,
        });
        self
    }

    /// Maps the stdout-streaming [`Terminal256`] at the conventional terminal address.
    pub fn terminal(self) -> Self {
        self.terminal_at(DEFAULT_TERM_BASE, DEFAULT_TERM_SIZE)
    }

    /// Maps the stdout-streaming [`Terminal256`] with an explicit address window.
    pub fn terminal_at(mut self, base: u16, size: u16) -> Self {
        self.terminal = Some((base, size));
        self
    }

    /// Presets a register before the first instruction executes.
    pub fn set_register(mut self, reg: Register, value: u16) -> Self {
        self.registers.push((reg, value));
        self
    }

    /// Convenience: presets `SP`, the stack grows down from this address.
    pub fn stack_top(self, addr: u16) -> Self {
        self.set_register(Register::SP, addr)
    }

    /// Enables the machine's built-in debug tracing.
    pub fn debug(mut self, enabled: bool) -> Self {
        self.debug = enabled;
        self
    }

    /// Performs the construction sequence and validates the whole configuration.
    pub fn build(self) -> Result<BuiltMachine, String> {
        if self.memory_size == 0 {
            return Err(String::from("memory size must be greater than zero"));
        }
        if self.memory_size > MAX_MEMORY {
            return Err(format!(
                "memory size of {} bytes exceeds the {MAX_MEMORY} bytes addressable with u16 addresses",
                self.memory_size
            ));
        }

        let mut linear = LinearMemory::new(self.memory_size);
        if let Some(program) = &self.program
            && !linear.write_program_at(self.program_base, program.instructions()) {
                return Err(String::from(
                    "failed to write the program into memory (does it fit? is the region free?)",
                ));
            }

        let mut memory = MemoryWithDevices::new(linear);

        let mut display = None;
        if let Some(config) = self.display {
            let (device, grid) = DisplayDevice::new(config.width, config.height);
            memory.register_device(device, config.base, config.size)?;
            display = Some(grid);
        }

        if let Some((base, size)) = self.terminal {
            memory.register_device(Terminal256::new(), base, size)?;
        }

        for (device, base, size) in self.devices {
            memory.register_device(device, base, size)?;
        }

        let mut machine = Machine::new_debug(memory, self.debug);
        // a loaded program starts executing at its load address; explicit
        // register presets are applied below and override this default
        if self.program.is_some() {
            machine.set_register(Register::PC, self.program_base);
        }
        for (reg, value) in self.registers {
            machine.set_register(reg, value);
        }

        Ok(BuiltMachine { machine, display })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust16vm::{machine::State, rv16asm};

    struct NullDevice;

    impl Device for NullDevice {
        fn read(&self, _offset: usize) -> u8 {
            0
        }

        fn write(&mut self, _offset: usize, _value: u8) {}
    }

    #[test]
    fn default_builder_builds_headless_machine() {
        let built = MachineBuilder::new().build().unwrap();

        assert!(built.display.is_none());
        assert_eq!(built.machine.register(Register::SP), 0);
    }

    #[test]
    fn memory_size_bounds_are_validated() {
        assert!(MachineBuilder::new().memory_size(0).build().is_err());
        assert!(MachineBuilder::new().memory_size(1 << 17).build().is_err());
        assert!(MachineBuilder::new().memory_size(1024).build().is_ok());
    }

    #[test]
    fn stack_top_sets_sp() {
        let built = MachineBuilder::new().stack_top(0xFFFF).build().unwrap();

        assert_eq!(built.machine.register(Register::SP), 0xFFFF);
    }

    #[test]
    fn program_from_bytes_matches_word_layout() {
        let program = Program::from_bytes(&[0x01, 0x02, 0x03]);

        assert_eq!(program.instructions(), &[0x0201, 0x0003]);
    }

    #[test]
    fn builds_and_runs_program_from_macro() {
        let built = MachineBuilder::new()
            .program(rv16asm! {
                "MOV A, #7",
                "ADD FLAGS, #1",
            })
            .build()
            .unwrap();
        let mut machine: Machine<MemoryWithDevices> = built.into();

        while let Ok(State::Continue) = machine.step() {}

        assert_eq!(machine.register(Register::A), 7);
    }

    #[test]
    fn display_receives_program_output() {
        let built = MachineBuilder::new()
            .program(rv16asm! {
                "MOV A, #72", // 'H'
                "MOV B, #0x0F",
                "MSL B, [#0 #7]",
                "MSL B, [#0 #5]", // B = 0xF000 (buffer start)
                "STB A, B",
                "MOV A, #3", // END_OF_TEXT marker
                "ADD B, #1",
                "STB A, B",
                "MOV C, #2",
                "MOV B, #0x0F",
                "MSL B, [#1 #4]",
                "MSL B, [#1 #7]",
                "MSL B, [#0 #1]", // B = 0xF102 (command port)
                "STB C, B",       // flush
                "ADD FLAGS, #1",  // halt
            })
            .display()
            .build()
            .unwrap();
        let grid = built.display.as_ref().unwrap().clone();
        let mut machine: Machine<MemoryWithDevices> = built.into();

        while let Ok(State::Continue) = machine.step() {}

        assert_eq!(grid.lock().unwrap().line(0).trim_end(), "H");
    }

    #[test]
    fn program_can_be_loaded_at_an_arbitrary_address() {
        let built = MachineBuilder::new()
            .memory_size(1024)
            .program_at(0x0100, rv16asm! {
                "MOV A, #9",
                "ADD FLAGS, #1",
            })
            .build()
            .unwrap();

        // execution starts at the load address
        assert_eq!(built.machine.register(Register::PC), 0x0100);
        let mut machine: Machine<MemoryWithDevices> = built.into();

        while let Ok(State::Continue) = machine.step() {}

        assert_eq!(machine.register(Register::A), 9);
    }

    #[test]
    fn explicit_pc_preset_overrides_the_program_base() {
        let built = MachineBuilder::new()
            .memory_size(1024)
            .program_at(0x0100, rv16asm! { "ADD FLAGS, #1" })
            .set_register(Register::PC, 0x0000)
            .build()
            .unwrap();

        assert_eq!(built.machine.register(Register::PC), 0x0000);
    }

    #[test]
    fn overlapping_devices_fail_to_build() {
        let result = MachineBuilder::new()
            .display()
            .device(NullDevice, DEFAULT_TERM_BASE, DEFAULT_TERM_SIZE)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn debug_mode_pauses_at_dbg_breakpoints() {
        let built = MachineBuilder::new()
            .program(rv16asm! {
                "MOV A, #1",
                "DBG", // breakpoint
                "MOV A, #2",
                "ADD FLAGS, #1",
            })
            .debug(true)
            .build()
            .unwrap();
        let mut machine: Machine<MemoryWithDevices> = built.into();

        assert_eq!(machine.step(), Ok(State::Continue)); // MOV A, #1
        assert_eq!(machine.step(), Ok(State::Debug)); // DBG: breakpoint
        assert_eq!(machine.step(), Ok(State::Continue)); // MOV A, #2
        // ADD FLAGS, #1 sets the halt bit but still returns Continue;
        // Stop is only observed on the next step
        assert_eq!(machine.step(), Ok(State::Continue));
        assert_eq!(machine.step(), Ok(State::Stop));
        assert_eq!(machine.register(Register::A), 2);
    }

    #[test]
    fn dbg_instructions_are_invisible_without_debug_mode() {
        let built = MachineBuilder::new()
            .program(rv16asm! {
                "MOV A, #1",
                "DBG",
                "MOV A, #2",
                "ADD FLAGS, #1",
            })
            .build()
            .unwrap();
        let mut machine: Machine<MemoryWithDevices> = built.into();

        // step() keeps returning Ok(Stop) on a halted machine, so stop
        // explicitly at the first Stop
        let mut states = vec![];
        while let Ok(state) = machine.step() {
            states.push(state);
            if state == State::Stop {
                break;
            }
        }

        assert_eq!(
            states,
            vec![State::Continue, State::Continue, State::Continue, State::Continue, State::Stop]
        );
        assert_eq!(machine.register(Register::A), 2);
    }
}
