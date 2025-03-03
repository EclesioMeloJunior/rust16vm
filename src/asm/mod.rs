use std::str::FromStr;

use crate::machine::{Instruction, Register};

#[derive(Debug)]
pub enum AsmError {
    InvalidRegister,
}

impl FromStr for Register {
    type Err = AsmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" => Ok(Register::A),
            "B" => Ok(Register::B),
            "C" => Ok(Register::C),
            "M" => Ok(Register::M),
            "PC" => Ok(Register::PC),
            "BP" => Ok(Register::BP),
            "SP" => Ok(Register::SP),
            "FLAGS" => Ok(Register::FLAGS),
            _ => Err(AsmError::InvalidRegister),
        }
    }
}

pub fn encode_instruction(inst: &Instruction) -> u16 {
    match inst {
        Instruction::Mov(reg, imm) => {
            let reg_code = (*reg as u16) & 0b111;
            let imm = imm & 0b111111111;
            (imm << 7) | (reg_code << 4) | 0b0001
        }
        Instruction::MovShift(reg, sh_amt, left_shift, imm) => {
            let reg_code = (*reg as u16) & 0b111;
            let shift = (*sh_amt as u16) & 0b111;
            let imm = imm & 0b11111;
            let mut dir = 0b0;
            if *left_shift {
                dir = 0b1;
            }

            (imm << 11) | dir << 10 | (shift << 7) | (reg_code << 4) | 0b0010
        }
        _ => 0,
    }
}

#[cfg(test)]
mod test {
    use crate::machine::{Instruction, Register};

    use super::encode_instruction;

    #[test]
    fn test_encode_instruction() {
        let mov = Instruction::Mov(Register::C, 10);
        let inst = encode_instruction(&mov);
        assert_eq!(0b0000010100100001, inst);

        let msl = Instruction::MovShift(Register::B, 4, false, 4);
        let inst = encode_instruction(&msl);
        assert_eq!(0b0010001000010010, inst);
    }
}
