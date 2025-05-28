#[allow(dead_code)]
use super::memory::Addressable;


#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Eq, Ord)]
#[repr(usize)]
pub enum Register {
    A,
    B,
    C,
    M,
    SP,
    PC,
    BP,

    // bit 0 - if set to 1 halts the machine
    // bit 1 - if set to 1 stores division module on stack
    // bit 2 - if set to 1 failed to perform memory write
    // bit 3 - if set to 1 then
    // [not eq|eq|less|less_eq|greater|greater_eq]
    // instruction is true
    FLAGS,
}

impl ToString for Register {
    fn to_string(&self) -> String {
        match self {
            Register::A => String::from("A"),
            Register::B => String::from("B"),
            Register::C => String::from("C"),
            Register::M => String::from("M"),
            Register::SP => String::from("SP"),
            Register::PC => String::from("PC"),
            Register::BP => String::from("BP"),
            Register::FLAGS => String::from("FLAGS"),
        }
    }
}

impl TryFrom<usize> for Register {
    type Error = String;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Register::A),
            1 => Ok(Register::B),
            2 => Ok(Register::C),
            3 => Ok(Register::M),
            4 => Ok(Register::SP),
            5 => Ok(Register::PC),
            6 => Ok(Register::BP),
            7 => Ok(Register::FLAGS),
            _ => Err(format!("invalid register: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum ArithmeticOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum CompareOp {
    Eq,        // 000
    NotEq,     // 001
    Less,      // 010
    LessEq,    // 011
    Greater,   // 100
    GreaterEq, // 101
}

impl TryFrom<usize> for CompareOp {
    type Error = String;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CompareOp::Eq),
            1 => Ok(CompareOp::NotEq),
            2 => Ok(CompareOp::Less),
            3 => Ok(CompareOp::LessEq),
            4 => Ok(CompareOp::Greater),
            5 => Ok(CompareOp::GreaterEq),
            _ => Err(format!("invalid compare opcode: {}", value)),
        }
    }
}

/// set of possible instructions
/// MOV - Move immediate to register
/// MSL | MSR - Move immediate to register shifting register value
/// ADD | SUB | MUL | Div - Arithmetic Operations
/// LDR | STR - operations on the memory
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Instruction {
    Noop,

    // Format: 0001 | reg(3) | immediate(9)
    Mov(Register, u16),

    // Move with shift
    // Format: 0010 | reg(3) | shift_amt(3) | direction(1) | immediate(5)
    // direction 1 then shift left
    // direction 0 then shift right
    MovShift(Register, u8, bool, u16),

    // Executes one of the arithmetic operations (add, sub, mul, div)
    // Format: 0011 | reg(3) | op(2) | src(1) | [src == 1]reg(3),[src == 0]imm(6)
    // op: 00 (add) 01 (sub) 10 (mul) 11 (div)
    Arith(Register, Option<Register>, Option<u16>, ArithmeticOp),

    // Load or Store a u16 the register value in the memory
    // Format: 0100 | reg(3) | reg(3) | type (1) | shift (5)
    LdrStr(Register, Register, bool, u8),

    // Load or Store a byte in the memory
    // Format: 1000 | reg(3) | reg(3) | type (1) | shift (5)
    LdbStb(Register, Register, bool, u8),

      // Copy a value stored in the first register to the
    // address in the second register
    // Format: 1001 | reg(3) | reg(3) | ...
    Cpy(Register, Register),

    // Set the PC to a specific memory address
    // Format: 0110 | mode (1) | [mode == 1]reg(3) | [mode == 0]imm(11)
    Jmp(Option<Register>, Option<u16>),

    // Reads FLAGS register at bit 3 and it its 1 then
    // sets PC register to the given memory address
    // Format: 0101 | mode (1) | [mode == 1]reg(3) | [mode == 0]imm(11)
    CondJmp(Option<Register>, Option<u16>),

    // Compare a base register with another register or with an immediate
    // Foarmat: 0111 | reg (3) | cmp (3) | mode (1) | [mode == 1] reg(3), [mode == 0]imm(5)
    Cmp(Register, Option<Register>, Option<u16>, CompareOp),

    // Jump to a specific address, but set the return instruction addr inside register RT
    // Foarmat: 1011 | ret (1) | address (11)
    CallRet(bool, u16),
}

impl TryFrom<u16> for Instruction {
    type Error = String;

    fn try_from(inst: u16) -> Result<Self, Self::Error> {
        let opcode = inst & 0b1111;
        match opcode {
            0b0000 => Ok(Instruction::Noop),
            0b0001 => {
                let reg_dst = ((inst >> 4) & 0b111) as usize;
                let imm = (inst >> 7) & 0b111111111;
                return Ok(Instruction::Mov(Register::try_from(reg_dst)?, imm));
            }
            0b0010 => {
                let reg_dst = ((inst >> 4) & 0b111) as usize;
                let sh_amt = ((inst >> 7) & 0b111) as u8;
                let dir = (inst >> 10) & 0b1 == 1;
                let imm = (inst >> 11) & 0b11111;
                return Ok(Instruction::MovShift(
                    Register::try_from(reg_dst)?,
                    sh_amt,
                    dir,
                    imm,
                ));
            }
            0b1001 => {
                let reg_src = ((inst >> 4) & 0b111) as usize;
                let reg_dst = ((inst >> 7) & 0b111) as usize;
                return Ok(Instruction::Cpy(
                    Register::try_from(reg_src)?,
                    Register::try_from(reg_dst)?,
                ));
            }
            0b0011 => {
                let reg_dst = Register::try_from(((inst >> 4) & 0b111) as usize)?;
                let op = match (inst >> 7) & 0b11 {
                    0b00 => ArithmeticOp::Add,
                    0b01 => ArithmeticOp::Sub,
                    0b10 => ArithmeticOp::Mul,
                    0b11 => ArithmeticOp::Div,
                    _ => unreachable!(),
                };

                let uses_reg_as_input = (inst >> 9) & 0b1 == 1;

                if uses_reg_as_input {
                    let reg_src = Register::try_from(((inst >> 10) & 0b111) as usize)?;
                    return Ok(Instruction::Arith(reg_dst, Some(reg_src), None, op));
                }

                let imm = inst >> 10;
                Ok(Instruction::Arith(reg_dst, None, Some(imm), op))
            }
            0b0100 => {
                let r0 = Register::try_from(((inst >> 4) & 0b111) as usize)?;
                let addr_reg = Register::try_from(((inst >> 7) & 0b111) as usize)?;
                let is_str = (inst >> 10) & 0b1 == 1;
                let shift = (inst >> 11) as u8;
                Ok(Instruction::LdrStr(r0, addr_reg, is_str, shift))
            }
            0b1000 => {
                let r0 = Register::try_from(((inst >> 4) & 0b111) as usize)?;
                let addr_reg = Register::try_from(((inst >> 7) & 0b111) as usize)?;
                let is_str = (inst >> 10) & 0b1 == 1;
                let shift = (inst >> 11) as u8;
                Ok(Instruction::LdbStb(r0, addr_reg, is_str, shift))
            }
            0b0110 => {
                let is_reg_mode = (inst >> 4) & 0b1 == 1;
                if is_reg_mode {
                    let reg = Register::try_from(((inst >> 5) & 0b111) as usize)?;
                    return Ok(Instruction::Jmp(Some(reg), None));
                }

                let imm = (inst >> 5) & 0b11111111111;
                Ok(Instruction::Jmp(None, Some(imm)))
            }
            0b0101 => {
                let is_reg_mode = (inst >> 4) & 0b1 == 1;
                if is_reg_mode {
                    let reg = Register::try_from(((inst >> 5) & 0b111) as usize)?;
                    return Ok(Instruction::Jmp(Some(reg), None));
                }

                let imm = (inst >> 5) & 0b11111111111;
                Ok(Instruction::CondJmp(None, Some(imm)))
            }
            0b0111 => {
                let reg = Register::try_from(((inst >> 4) & 0b111) as usize)?;
                let cmp = ((inst >> 7) & 0b111) as usize;
                let reg_mode = (inst >> 10) & 0b1 == 1;
                let (opt_reg, opt_imm) = if reg_mode {
                    let src_reg = Register::try_from(((inst >> 11) & 0b111) as usize)?;
                    (Some(src_reg), None)
                } else {
                    let imm = (inst >> 11) & 0b11111;
                    (None, Some(imm))
                };

                Ok(Instruction::Cmp(
                    reg,
                    opt_reg,
                    opt_imm,
                    CompareOp::try_from(cmp)?,
                ))
            }
            0b1011 => {
                let is_ret= ((inst >> 4) & 0b1) == 1;
                Ok(Instruction::CallRet(is_ret, (inst >> 5) as u16))
            }
            _ => Err(format!("unexpected instruction: {:#06x}", inst)),
        }
    }
}

pub struct Machine<M: Addressable> {
    registers: [u16; 8],
    memory: M,
}

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub enum State {
    Continue,
    Stop
}

impl<M: Addressable> Machine<M> {
    pub fn new(mem: M) -> Self {
        Self {
            registers: [0; 8],
            memory: mem,
        }
    }

    pub fn set_register(&mut self, reg: Register, value: u16) {
        self.registers[reg as usize] = value;
    }

    pub fn step(&mut self) -> Result<State, String> {
        let halt = self.registers[Register::FLAGS as usize] & 0b1 == 1;
        if halt {
            return Ok(State::Stop);
        }

        let pc = self.registers[Register::PC as usize];
        let raw = self.memory.read2(pc).unwrap();

        let inst = Instruction::try_from(raw)?;
        
        self.print_regs();
        println!("{:?} @ {}", inst, pc);

        match inst {
            Instruction::Mov(dst_reg, imm) => {
                self.registers[dst_reg as usize] = imm;
            }
            Instruction::MovShift(dst_reg, sh_am, left, imm) => {
                let mut curr_value = self.registers[dst_reg as usize];
                if left {
                    curr_value <<= sh_am;
                } else {
                    curr_value >>= sh_am;
                }

                curr_value |= imm;
                self.registers[dst_reg as usize] = curr_value;
            }
            Instruction::Cpy(reg_src, reg_dst) => {
                let src_addr = self.registers[reg_src as usize];
                let dst_addr = self.registers[reg_dst as usize];
                if !self.memory.copy(src_addr, dst_addr, 1) {
                    self.set_flags((0b1 << 2) | 0b1);
                }
            }
            Instruction::Arith(dst_reg, src_reg, imm, arith_op) => match (src_reg, imm) {
                (Some(src), None) => {
                    self.arithmetic_op(dst_reg, self.registers[src as usize], arith_op)
                }
                (None, Some(imm)) => self.arithmetic_op(dst_reg, imm, arith_op),
                _ => unreachable!(),
            },
            Instruction::LdrStr(r0, addr_reg, is_str, shift) => {
                let base = self.registers[addr_reg as usize];
                let at = base + (shift as u16);

                if is_str {
                    let to_store = self.registers[r0 as usize];
                    if !self.memory.write2(at, to_store) {
                        // failed to perform memory write
                        // force a halt
                        // 0b0...101
                        self.set_flags((0b1 << 2) | 0b1);
                    }
                } else {
                    println!("ONDE ESTOU LENDO: {}", at);
                    if let Some(value) = self.memory.read2(at) {
                        self.registers[r0 as usize] = value;
                    }
                }
            }
            Instruction::LdbStb(r0, addr_reg, is_str, shift) => {
                let base = self.registers[addr_reg as usize];
                let at = base + (shift as u16);

                if is_str {
                    let to_store: u8 = self.registers[r0 as usize] as u8;
                    if !self.memory.write(at, to_store) {
                        // failed to perform memory write
                        // force a halt
                        // 0b0...101
                        self.set_flags((0b1 << 2) | 0b1);
                    }
                } else {
                    if let Some(value) = self.memory.read(at) {
                        self.registers[r0 as usize] = value as u16;
                    }
                }
            }
            Instruction::Jmp(opt_reg, opt_imm) => match (opt_reg, opt_imm) {
                (Some(reg), None) => {
                    let addr = self.registers[reg as usize];
                    self.registers[Register::PC as usize] = addr;
                    return Ok(State::Continue);
                }
                (None, Some(imm)) => {
                    self.registers[Register::PC as usize] = imm;
                    return Ok(State::Continue);
                }
                _ => unreachable!(),
            },
            Instruction::CondJmp(opt_reg, opt_imm) => {
                let curr_flags = self.registers[Register::FLAGS as usize];
                let should_jmp = (curr_flags >> 3) & 0b1 == 1;
                if should_jmp {
                    // switch back the bit to 0 after reading it
                    self.registers[Register::FLAGS as usize] ^= 1 << 3;

                    match (opt_reg, opt_imm) {
                        (Some(reg), None) => {
                            let addr = self.registers[reg as usize];
                            self.registers[Register::PC as usize] = addr;
                            return Ok(State::Continue);
                        }
                        (None, Some(imm)) => {
                            self.registers[Register::PC as usize] = imm;
                            return Ok(State::Continue);
                        }
                        _ => unreachable!(),
                    }
                }
            }
            Instruction::Cmp(reg, opt_reg, opt_imm, op) => {
                let lhs = self.registers[reg as usize];
                match (opt_reg, opt_imm) {
                    (Some(other_reg), None) => {
                        self.compare_op(lhs, self.registers[other_reg as usize], op)
                    }
                    (None, Some(imm)) => self.compare_op(lhs, imm, op),
                    _ => unreachable!(),
                };
            }
            Instruction::CallRet(is_ret, addr) => {
                if is_ret {
                    self.registers[Register::PC as usize] = self.registers[Register::M as usize];
                    return Ok(State::Continue);
                } 

                let curr = self.registers[Register::PC as usize];

                self.registers[Register::PC as usize] = addr;
                self.registers[Register::M as usize] = curr + 2;

                return Ok(State::Continue);
            }
            _ => return Err(format!("invalid instruction: {:?}", inst)),
        }

        self.registers[Register::PC as usize] += 2;
        Ok(State::Continue)
    }

    fn set_flags(&mut self, flags: u16) {
        self.registers[Register::FLAGS as usize] |= flags;
    }

    fn remove_flags(&mut self, flags: u16) {
        self.registers[Register::FLAGS as usize] ^= flags;
    }

    fn is_flag_active(&mut self, idx: u16) -> bool {
        let flags = self.registers[Register::FLAGS as usize];
        let extract_flag = flags & (1 << idx);
        return extract_flag != 0;
    }

    fn arithmetic_op(&mut self, dst_reg: Register, imm: u16, op: ArithmeticOp) {
        let lhs = self.registers[dst_reg as usize];
        let result = match op {
            ArithmeticOp::Add => lhs + imm,
            ArithmeticOp::Sub => lhs - imm,
            ArithmeticOp::Mul => lhs * imm,
            ArithmeticOp::Div => {
                let store_mod = (self.registers[Register::FLAGS as usize] >> 1) & 0b1 == 1;
                if store_mod {
                    self.memory
                        .write(self.registers[Register::SP as usize], (lhs % imm) as u8);
                }
                lhs / imm
            }
        };

        self.registers[dst_reg as usize] = result;
    }

    fn compare_op(&mut self, lhs: u16, rhs: u16, op: CompareOp) {
        let is_true = match op {
            CompareOp::Eq => lhs == rhs,
            CompareOp::NotEq => lhs != rhs,
            CompareOp::Less => lhs < rhs,
            CompareOp::LessEq => lhs <= rhs,
            CompareOp::Greater => lhs > rhs,
            CompareOp::GreaterEq => lhs >= rhs,
        };

        if is_true {
            self.set_flags(1 << 3);
        } else if self.is_flag_active(3) {
            self.remove_flags(1 << 3);            
        }
    }

    pub fn print_regs(&self) -> () {
        for (idx, value) in self.registers.iter().enumerate() {
            println!(
                "{}:\t{:#018b} | {:#04x} | {}",
                Register::try_from(idx).unwrap().to_string(),
                value,
                value,
                value
            );
        }
    }
}

#[cfg(test)]
mod test {
    use std::usize;

    use crate::{
        machine::{Register, State},
        memory::{Addressable, LinearMemory},
        rv16asm,
    };

    use super::Machine;

    #[test]
    fn invalid_instruction_opcode() {
        let mut mem = LinearMemory::new(1024);
        mem.write2(0, 0 as u16);
        let mut machine = Machine::new(mem);

        let result = machine.step();
        assert!(result.is_err());
        assert_eq!(Err(String::from("invalid instruction: Noop")), result);
    }

    #[test]
    fn valid_mov_instruction() {
        let mut mem = LinearMemory::new(8 * 1024); //8Kb
        // 3 instructions to fill a register with ones
        mem.write2(0, 0b1111111110000001); // MOV A, #8
        mem.write2(2, 0b1111111010000010); // MSL A, 5 #31
        mem.write2(4, 0b0001110100000010); // MSL A, 2 #3

        let mut machine = Machine::new(mem);
        for _i in 0..3 {
            machine.step().unwrap();
        }

        assert_eq!(machine.registers[Register::A as usize], u16::MAX);
    }

    #[test]
    fn arithmetic_instruction() {
        let default_mem = || {
            let mut mem = LinearMemory::new(8 * 1024); //8Kb
            mem.write2(0, 0b0000010000000001); // MOV A, #8
            mem
        };

        // run adds number 8 to register A
        let run =
            move |instrs: Vec<u16>, assertions: Vec<(Register, u16)>| -> Machine<LinearMemory> {
                let mut mem = default_mem();
                let inst_len = instrs.len();

                for (idx, inst) in instrs.iter().enumerate() {
                    assert!(mem.write2(((idx + 1) * 2) as u16, *inst));
                }

                let mut machine = Machine::new(mem);
                machine.set_register(Register::SP, 8 * 1024);
                for _i in 0..inst_len + 1 {
                    machine.step().unwrap();
                }

                for (reg, exp) in assertions {
                    assert_eq!(machine.registers[reg as usize], exp);
                }

                machine
            };

        // ADD A, #2
        run(vec![0b0000100000000011], vec![(Register::A, 10)]);

        // MOV B, #2
        // ADD A, B
        run(
            vec![0b000000010_001_0001, 0b000_001_1_00_000_0011],
            vec![(Register::A, 10)],
        );

        // SUB A, #2
        run(vec![0b000010_0_01_000_0011], vec![(Register::A, 6)]);

        // MOV B, #2
        // SUB A, B
        run(
            vec![0b000000010_001_0001, 0b000_001_1_01_000_0011],
            vec![(Register::A, 6)],
        );

        // MUL A, #2
        run(vec![0b000010_0_10_000_0011], vec![(Register::A, 16)]);

        // MOV B, #2
        // MUL A, B
        run(
            vec![0b000000010_001_0001, 0b000_001_1_10_000_0011],
            vec![(Register::A, 16)],
        );

        // DIV A, #2 - no FLAGS
        run(vec![0b000010_0_11_000_0011], vec![(Register::A, 4)]);

        // MOV FL, #2
        // SUB SP, #2
        // DIV A, #5 - with FLAGS
        let machine = run(
            vec![
                0b000000010_111_0001, 
                0b000010_0_01_100_0011,
                0b000101_0_11_000_0011,
            ],
            vec![(Register::A, 1)],
        );
        let stored = machine
            .memory
            .read2(machine.registers[Register::SP as usize]);

        assert_eq!(stored.unwrap(), 3_u16);

        // MOV B, #2
        // DIV A, B - no FLAGS
        run(
            vec![0b000000010_001_0001, 0b000_001_1_11_000_0011],
            vec![(Register::A, 4)],
        );

        // MOV FL, #2
        // SUB SP, #2
        // MOV B, #2
        // DIV A, B - with FLAGS
        let machine = run(
            vec![
                0b000000010_111_0001,
                0b000010_0_01_100_0011,
                0b000000101_001_0001,
                0b000_001_1_11_000_0011,
            ],
            vec![(Register::A, 1)],
        );
        let stored = machine
            .memory
            .read2(machine.registers[Register::SP as usize]);

        assert_eq!(stored.unwrap(), 3_u16);
    }

    #[test]
    fn test_jump_instruction() {
        let mut mem = LinearMemory::new(1024);
        mem.write2(0, 0b00000001010_0_0110); // JMP #10
        mem.write2(10, 0b0000010000000001); // MOV A, #8

        let mut machine = Machine::new(mem);
        machine.print_regs();
        machine.step().unwrap();
        assert_eq!(machine.registers[Register::PC as usize], 10);

        machine.print_regs();
        machine.step().unwrap();
        machine.print_regs();

        assert_eq!(machine.registers[Register::A as usize], 8);
        assert_eq!(machine.registers[Register::PC as usize], 12);
    }

    #[test]
    fn a_simple_for_loop() {
        let program = rv16asm! {
            "MOV A, #0",

            // loop
            "EQ A, #10",
            "CJP #10",
            "ADD A, #1",
            "JMP #2",

            "ADD FLAGS, #1" // halts machine
        };

        let mut mem = LinearMemory::new(1024);
        assert!(mem.write_program(&program));

        let mut machine = Machine::new(mem);
        while let Ok(State::Continue) = machine.step()  {
            machine.print_regs();
        }
        assert_eq!(machine.registers[Register::A as usize], 10);
    }

    #[test]
    fn should_halt_trying_to_write_at_read_only_addr() {
        let program = rv16asm! {
            "MOV A, #39",
            "MOV B, #100", // B stores the addr
            "STR A, B"
        };

        let mut mem = LinearMemory::new(1024);
        mem.as_read_only(100, 2); // defines addr 100 as readonly
        assert!(mem.write_program(&program));

        let mut machine = Machine::new(mem);
        while let Ok(State::Continue) = machine.step()  {
            machine.print_regs();
        }

        assert_eq!(machine.registers[Register::FLAGS as usize], 5) // the FLAGS should be 0...101
    }

    #[test]
    fn should_change_flags() {
         let program = rv16asm! {
            "MOV A, #1",
            "EQ A, #1",
            "EQ A, #2",
        };

        let mut mem = LinearMemory::new(1024);
        assert!(mem.write_program(&program));

        let mut machine = Machine::new(mem);
        
        machine.step().unwrap(); // MOV A, #1

        machine.step().unwrap(); // EQ A, #1
        assert_eq!(machine.registers[Register::FLAGS as usize], 0b1000);  // the FLAGS should be 0...101

        machine.step().unwrap(); // EQ A, #2
        assert_eq!(machine.registers[Register::FLAGS as usize], 0b0000) // the FLAGS should be 0...101
    }

    #[test]
    fn run_factorial_algorithm() {
        let program = rv16asm! {
            "MOV B, #8",
            "MOV A, #1",

            "LTE B, #1",
            "CJP #14",
            "MUL A, B",
            "SUB B, #1",
            "JMP #4",

            "ADD FLAGS, #1",
        };

        let mut mem = LinearMemory::new(1024);
        assert!(mem.write_program(&program));

        let mut machine = Machine::new(mem);
        while let Ok(State::Continue) = machine.step()  {
        }

        assert_eq!(machine.registers[Register::A as usize], 40_320);
    }

    #[test]
    fn call_to_another_label() {
        /*
        func sum(a, b int) int {
            return a + b
        }

        func main() {
            sum(1, 2);
            c := 55;
        }
        */        
        let program = rv16asm! {
            "ADD A, B",
            "RET",
            
            "MOV C, #11",

            "MOV A, #1",
            "MOV B, #2",

            "CALL #0",

            "MOV C, #55",
            "ADD FLAGS, #1"
        };

        println!("{:?}", program);

        let mut mem = LinearMemory::new(1024);
        assert!(mem.write_program(&program));

        let mut machine = Machine::new(mem);

        machine.set_register(Register::PC, 6);

        while let Ok(State::Continue) = machine.step() {
        }
        
        assert_eq!(Ok(State::Stop), machine.step());
        assert_eq!(machine.registers[Register::A as usize], 3);
        assert_eq!(machine.registers[Register::B as usize], 2);
        assert_eq!(machine.registers[Register::C as usize], 55);
    }


    #[test]
    fn run_fibonacci_algorithm() {
        let program = rv16asm! {
            "MOV A, #0",
            "MOV B, #1",
            "MOV M, #0",

            "GTE M, #9",
            "CJP #40",
            "ADD A, B",

            "SUB SP, #2",
            "STR A, SP",
            "LDR C, SP",
            "ADD SP, #2",

            "SUB SP, #2",
            "STR B, SP",
            "LDR A, SP",
            "ADD SP, #2",

            "SUB SP, #2",
            "STR C, SP",
            "LDR B, SP",
            "ADD SP, #2",

            "ADD M, #1",
            "JMP #6",
            
            "ADD FLAGS, #1",
        };

        let mut mem = LinearMemory::new(1024);
        assert!(mem.write_program(&program));

        let mut machine = Machine::new(mem);
        machine.set_register(Register::SP, 1024);

        while let Ok(State::Continue) = machine.step() {
        }

        machine.print_regs();
        
        assert_eq!(machine.registers[Register::A as usize], 34);
        assert_eq!(machine.registers[Register::B as usize], 55);
        assert_eq!(machine.registers[Register::M as usize], 9);
        assert_eq!(machine.registers[Register::FLAGS as usize], 1) // the FLAGS should be 0...101
    }

    #[test]
    fn test_mod_operation() {
        let program = rv16asm! {
            "MOV A, #24",

            "ADD FLAGS, #2",
            
            "SUB SP, #1",
            "DIV A, #12",
            "LDB C, SP",
            "ADD SP, #1",

            "ADD FLAGS, #1",
        };
        let mut mem = LinearMemory::new(1024);
        assert!(mem.write_program(&program));

        let mut machine = Machine::new(mem);
        machine.set_register(Register::SP, 100);

        while let Ok(State::Continue) = machine.step() {
        }

        machine.print_regs();

        assert_eq!(machine.registers[Register::A as usize], 2);
        assert_eq!(machine.registers[Register::C as usize], 0);
        assert_eq!(machine.registers[Register::SP as usize], 100);
        assert_eq!(machine.registers[Register::FLAGS as usize], 0b11);
    }
}
