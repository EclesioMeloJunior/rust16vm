use std::str::FromStr;

use crate::machine::{ArithmeticOp, Instruction, Register};

#[derive(Debug)]
pub enum AsmError {
    InvalidRegister,
    InvalidOperands,
    InvalidInstruction,
    InvalidFormat,
    InvalidImmediate,
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

pub fn encode_instruction(inst: &Instruction) -> Result<u16, AsmError> {
    match inst {
        Instruction::Mov(reg, imm) => {
            let reg_code = (*reg as u16) & 0b111;
            let imm = imm & 0b111111111;
            Ok((imm << 7) | (reg_code << 4) | 0b0001)
        }
        Instruction::MovShift(reg, sh_amt, left_shift, imm) => {
            let reg_code = (*reg as u16) & 0b111;
            let shift = (*sh_amt as u16) & 0b111;
            let imm = imm & 0b11111;
            let mut dir = 0b0;
            if *left_shift {
                dir = 0b1;
            }

            Ok((imm << 11) | dir << 10 | (shift << 7) | (reg_code << 4) | 0b0010)
        }
        Instruction::Arith(dst_reg, opt_src_reg, opt_imm, op) => {
            let reg_code = (*dst_reg as u16) & 0b111;
            let op = match op {
                ArithmeticOp::Add => 0b00,
                ArithmeticOp::Sub => 0b01,
                ArithmeticOp::Mul => 0b10,
                ArithmeticOp::Div => 0b11,
            };

            let (src, rhs) = match (opt_src_reg, opt_imm) {
                (Some(reg), None) => (0b1, (*reg as u16) & 0b111),
                (None, Some(imm)) => (0b0, imm & 0b111111),
                _ => return Err(AsmError::InvalidOperands),
            };

            Ok((rhs << 10) | (src << 9) | (op << 7) | (reg_code << 4) | 0b0011)
        }
        Instruction::LdrStr(r0, addr_reg, is_str, shift) => {
            let r0_code = (*r0 as u16) & 0b111;
            let addr_reg = (*addr_reg as u16) & 0b111;
            let shift = (*shift as u16) & 0b11111;
            let mut str = 0b0;
            if *is_str {
                str = 0b1;
            }

            Ok((shift << 11) | (str << 10) | (addr_reg << 7) | (r0_code << 4) | 0b0100)
        }
        _ => Err(AsmError::InvalidInstruction),
    }
}

type ParserFn = Box<dyn Fn(&[&str]) -> Result<Instruction, AsmError>>;

pub fn parse_assembly_line(line: &str) -> Result<Instruction, AsmError> {
    let line = line.trim();
    if line.is_empty() || line.starts_with(";") {
        return Err(AsmError::InvalidFormat);
    }

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Err(AsmError::InvalidFormat);
    }

    let instruction = parts[0].to_uppercase();
    let parser: ParserFn = match instruction.as_str() {
        "MOV" => Box::new(parse_mov),
        "MSL" => Box::new(parse_mov_shift(true)),
        "MSR" => Box::new(parse_mov_shift(false)),
        _ => return Err(AsmError::InvalidInstruction),
    };

    parser(&parts[1..])
}

fn parse_immediate(s: &str) -> Result<u16, AsmError> {
    if s.starts_with("#") {
        let value = &s[1..];
        return value.parse::<u16>().map_err(|_| AsmError::InvalidImmediate);
    }

    Err(AsmError::InvalidImmediate)
}

fn parse_mov(args: &[&str]) -> Result<Instruction, AsmError> {
    if args.len() != 2 {
        return Err(AsmError::InvalidInstruction);
    }

    let reg = args[0].trim_end_matches(',').parse::<Register>()?;
    let imm = parse_immediate(args[1])?;

    Ok(Instruction::Mov(reg, imm))
}

fn parse_mov_shift(dir: bool) -> impl Fn(&[&str]) -> Result<Instruction, AsmError> {
    move |args: &[&str]| -> Result<Instruction, AsmError> {
        if args.len() != 3 {
            return Err(AsmError::InvalidInstruction);
        }

        let reg = args[0].trim_end_matches(',').parse::<Register>()?;

        let value = parse_immediate(args[1].trim_start_matches("["))?;
        let shift = parse_immediate(args[2].trim_end_matches("]"))?;

        Ok(Instruction::MovShift(reg, shift.try_into().map_err(|_| AsmError::InvalidInstruction)?, dir, value))
    }
}

#[cfg(test)]
mod test {
    use crate::machine::{ArithmeticOp, Instruction, Register};

    use super::{encode_instruction, parse_assembly_line};

    #[test]
    fn test_encode_instruction() {
        let mov = Instruction::Mov(Register::C, 10);
        let inst = encode_instruction(&mov).unwrap();
        assert_eq!(0b0000010100100001, inst);

        let msl = Instruction::MovShift(Register::B, 4, true, 4);
        let inst = encode_instruction(&msl).unwrap();
        assert_eq!(0b0010011000010010, inst);

        let msr = Instruction::MovShift(Register::B, 4, false, 4);
        let inst = encode_instruction(&msr).unwrap();
        assert_eq!(0b0010001000010010, inst);

        let add = Instruction::Arith(Register::B, Some(Register::C), None, ArithmeticOp::Add);
        let inst = encode_instruction(&add).unwrap();
        assert_eq!(0b0000101000010011, inst);

        let add_imm = Instruction::Arith(Register::C, None, Some(8), ArithmeticOp::Div);
        let inst = encode_instruction(&add_imm).unwrap();
        assert_eq!(0b0010000110100011, inst);

        let str = Instruction::LdrStr(Register::B, Register::SP, true, 0);
        let inst = encode_instruction(&str).unwrap();
        assert_eq!(0b0000011000010100, inst);

        let ldr = Instruction::LdrStr(Register::B, Register::A, false, 10);
        let inst = encode_instruction(&ldr).unwrap();
        assert_eq!(0b0101000000010100, inst);
    }

    #[test]
    fn test_assembly_line() {
        let input = "MOV A, #10";
        let inst = parse_assembly_line(input).unwrap();
        assert_eq!(inst, Instruction::Mov(Register::A, 10))

        let input = "MSL A, [#11 #4]"
    }
}
