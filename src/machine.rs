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
    FLAGS,
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Register::A => write!(f, "Reg A"),
            Register::B => write!(f, "Reg B"),
            Register::C => write!(f, "Reg C"),
            Register::M => write!(f, "Reg M"),
            Register::SP => write!(f, "Reg SP"),
            Register::PC => write!(f, "Reg PC"),
            Register::BP => write!(f, "Reg BP"),
            Register::FLAGS => write!(f, "Reg FL"),
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

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ArithmeticOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// set of possible instructions
/// MOV - Move immediate to register
/// MSL | MSR - Move immediate to register shifting register value
/// ADD | SUB | MUL | Div - Arithmetic Operations
/// LDR | STR - operations on the memory
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Instruction {
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

    // Load or Store the register value in the memory
    // Format: 0100 | reg(3) | reg(3) | type (1) | shift (5)
    LdrStr(Register, Register, bool, u8),
}

impl TryFrom<u16> for Instruction {
    type Error = String;

    fn try_from(inst: u16) -> Result<Self, Self::Error> {
        let opcode = inst & 0b1111;
        match opcode {
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
            _ => Err(format!("unexpected instruction: {:#06x}", inst)),
        }
    }
}

pub struct Machine<M: Addressable> {
    registers: [u16; 8],
    memory: M,
}

impl<M: Addressable> Machine<M> {
    pub fn new(mem: M) -> Self {
        Self {
            registers: [0; 8],
            memory: mem,
        }
    }

    pub fn step(&mut self) -> Result<(), String> {
        let halt = self.registers[Register::FLAGS as usize] & 0b1 == 1;
        if halt {
            return Ok(());
        }

        let pc = self.registers[Register::PC as usize];
        let raw = self.memory.read2(pc).unwrap();

        let inst = Instruction::try_from(raw)?;
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
                    if let Some(value) = self.memory.read2(at) {
                        self.registers[r0 as usize] = value;
                    }
                }
            }
        }

        self.registers[Register::PC as usize] += 2;
        Ok(())
    }

    fn set_flags(&mut self, flags: u16) {
        self.registers[Register::FLAGS as usize] |= flags;
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
                        .write2(self.registers[Register::SP as usize], lhs % imm);
                    self.registers[Register::SP as usize] += 2;
                }
                lhs / imm
            }
        };

        self.registers[dst_reg as usize] = result;
    }

    pub fn print_regs(&self) -> () {
        for (idx, value) in self.registers.iter().enumerate() {
            println!(
                "{}:\t{:#018b} | {:#04x} | {}",
                Register::try_from(idx).unwrap(),
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
        machine::Register,
        memory::{Addressable, LinearMemory},
    };

    use super::Machine;

    #[test]
    fn invalid_instruction_opcode() {
        let mut mem = LinearMemory::new(1024);
        mem.write2(0, 0 as u16);
        let mut machine = Machine::new(mem);

        let result = machine.step();
        assert!(result.is_err());
        assert_eq!(Err(String::from("unexpected instruction: 0x0000")), result);
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
        // DIV A, #5 - with FLAGS
        let machine = run(
            vec![0b000000010_111_0001, 0b000101_0_11_000_0011],
            vec![(Register::A, 1)],
        );
        let stored = machine
            .memory
            .read2(machine.registers[Register::SP as usize] - 2_u16);

        assert_eq!(stored.unwrap(), 3_u16);

        // MOV B, #2
        // DIV A, B - no FLAGS
        run(
            vec![0b000000010_001_0001, 0b000_001_1_11_000_0011],
            vec![(Register::A, 4)],
        );

        // MOV FL, #2
        // MOV B, #2
        // DIV A, B - with FLAGS
        let machine = run(
            vec![
                0b000000010_111_0001,
                0b000000101_001_0001,
                0b000_001_1_11_000_0011,
            ],
            vec![(Register::A, 1)],
        );
        let stored = machine
            .memory
            .read2(machine.registers[Register::SP as usize] - 2_u16);

        assert_eq!(stored.unwrap(), 3_u16);
    }
}
