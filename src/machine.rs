use std::usize;

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

#[derive(Debug)]
pub enum Instruction {
    // Move immediate to register
    // Format: 0001 | reg(3) | immediate(9)
    Mov(Register, u16),

    // Move with shift
    // Format: 0010 | reg(3) | shift_amt(3) | direction(1) | immediate(5)
    // direction 1 then shift left
    // direction 0 then shift right
    MovShift(Register, u8, bool, u16),
    // Add more instructions here...
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
            _ => Err(format!("Unexpected instruction: {:#06x}", inst)),
        }
    }
}

impl Instruction {
    pub fn from_u16(inst: u16) -> Result<Self, String> {
        let opcode = inst & 0b1111;

        match opcode {
            0b0001 => {
                let reg_idx = ((inst >> 4) & 0b111) as usize;
                let reg = match reg_idx {
                    0 => Register::A,
                    1 => Register::B,
                    2 => Register::C,
                    3 => Register::M,
                    4 => Register::SP,
                    5 => Register::PC,
                    6 => Register::BP,
                    7 => Register::FLAGS,
                    _ => return Err(format!("Invalid register index: {}", reg_idx)),
                };
                let imm = (inst >> 7) & 0b111111111;
                Ok(Instruction::Mov(reg, imm))
            }
            0b0010 => {
                let reg_idx = ((inst >> 4) & 0b111) as usize;
                let reg = match reg_idx {
                    0 => Register::A,
                    1 => Register::B,
                    2 => Register::C,
                    3 => Register::M,
                    4 => Register::SP,
                    5 => Register::PC,
                    6 => Register::BP,
                    7 => Register::FLAGS,
                    _ => return Err(format!("Invalid register index: {}", reg_idx)),
                };
                let shift_amt = ((inst >> 7) & 0b111) as u8;
                let direction = ((inst >> 10) & 0b1) == 1; // true = left, false = right
                let imm = (inst >> 11) & 0b11111;
                Ok(Instruction::MovShift(reg, shift_amt, direction, imm))
            }
            // Add more opcodes here
            _ => Err(format!("Unexpected instruction: {:#06x}", inst)),
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
        let pc = self.registers[Register::PC as usize];
        let raw = self.memory.read2(pc).unwrap();

        let inst = Instruction::try_from(raw)?;
        println!("{:?} @ {}", inst, pc);

        match inst {
            Instruction::Mov(dst_reg, imm) => {
                self.registers[dst_reg as usize] = imm;
            },
            Instruction::MovShift(dst_reg, sh_am, left, imm) => {
                let mut curr_value = self.registers[dst_reg as usize];
                if left {
                    curr_value <<=sh_am;
                } else {
                    curr_value >>= sh_am;
                }

                curr_value |= imm;
                self.registers[dst_reg as usize] = curr_value;
            },
        }

        self.registers[Register::PC as usize] += 2;
        Ok(())
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
