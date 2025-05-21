use crate::machine::{ArithmeticOp, CompareOp, Instruction, Register};
use std::{
    collections::{HashMap, hash_map::Entry},
    env::args,
    fmt::format,
    hash::Hash,
    str::FromStr,
};

pub mod macros;

#[derive(Debug)]
pub enum AsmError {
    InvalidRegister,
    InvalidOperands,
    InvalidInstruction,
    InvalidFormat,
    InvalidImmediate,
    UnresolvedLabel(String),
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

impl ToString for Instruction {
    fn to_string(&self) -> String {
        match self {
            Instruction::Noop => "NOOP".to_string(),
            Instruction::Mov(reg, val) => {
                format!("MOV {}, #{}", reg.to_string(), val.to_string())
            }
            Instruction::MovShift(reg, shift_amt, is_left, imm) => {
                if *is_left {
                    format!("MSL {}, [#{} #{}]", reg.to_string(), shift_amt, imm)
                } else {
                    format!("MSR {}, [#{} #{}]", reg.to_string(), shift_amt, imm)
                }
            }
            Instruction::Cpy(src_reg, dst_reg) => {
                format!("CPY {}, {}", src_reg.to_string(), dst_reg.to_string())
            }
            Instruction::Arith(reg, opt_reg, opt_imm, op) => {
                let op = match op {
                    ArithmeticOp::Add => "ADD",
                    ArithmeticOp::Sub => "SUB",
                    ArithmeticOp::Mul => "MUL",
                    ArithmeticOp::Div => "DIV",
                };

                let operand = match (opt_reg, opt_imm) {
                    (Some(reg), None) => reg.to_string(),
                    (None, Some(imm)) => format!("#{}", imm.to_string()),
                    _ => format!("undefined"),
                };

                format!("{} {}, {}", op.to_string(), reg.to_string(), operand)
            }
            Instruction::LdrStr(reg, rhs_reg, is_str, shift) => {
                let op = if *is_str { "STR" } else { "LDR" };

                let rhs = if *shift == 0 {
                    rhs_reg.to_string()
                } else {
                    format!("[{} #{}]", rhs_reg.to_string(), shift.to_string())
                };

                format!(
                    "{} {}, {}",
                    op.to_string(),
                    reg.to_string(),
                    rhs.to_string(),
                )
            }
            Instruction::LdbStb(reg, rhs_reg, is_str, shift) => {
                let op = if *is_str { "STB" } else { "LDB" };

                let rhs = if *shift == 0 {
                    rhs_reg.to_string()
                } else {
                    format!("[{} #{}]", rhs_reg.to_string(), shift.to_string())
                };

                format!(
                    "{} {}, {}",
                    op.to_string(),
                    reg.to_string(),
                    rhs.to_string(),
                )
            }
            Instruction::Jmp(opt_reg, opt_imm) => {
                let src = match (opt_reg, opt_imm) {
                    (Some(reg), None) => reg.to_string(),
                    (None, Some(imm)) => format!("#{}", imm.to_string()),
                    _ => "undefined".to_string(),
                };

                format!("JMP {}", src)
            }
            Instruction::CondJmp(opt_reg, opt_imm) => {
                let src = match (opt_reg, opt_imm) {
                    (Some(reg), None) => reg.to_string(),
                    (None, Some(imm)) => format!("#{}", imm.to_string()),
                    _ => "undefined".to_string(),
                };

                format!("CJP {}", src)
            }
            Instruction::Cmp(reg, opt_reg, opt_imm, op) => {
                let op = match op {
                    CompareOp::Eq => "EQ",
                    CompareOp::NotEq => "NEQ",
                    CompareOp::Less => "LT",
                    CompareOp::LessEq => "LTE",
                    CompareOp::Greater => "GT",
                    CompareOp::GreaterEq => "GTE",
                };

                let operand = match (opt_reg, opt_imm) {
                    (Some(reg), None) => reg.to_string(),
                    (None, Some(imm)) => format!("#{}", imm.to_string()),
                    _ => format!("undefined"),
                };

                format!("{} {}, {}", op.to_string(), reg.to_string(), operand)
            }
            Instruction::CallRet(is_ret, addr) => {
                if *is_ret {
                    return "RET".to_string();
                }

                format!("CALL #{}", addr.to_string())
            }
        }
    }
}

pub fn encode_instruction(inst: &Instruction) -> u16 {
    match inst {
        Instruction::Noop => 0,
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
        Instruction::Cpy(src, dst) => {
            let src_reg = (*src as u16) & 0b111;
            let dst_reg = (*dst as u16) & 0b111;
            (dst_reg << 7) | (src_reg << 4) | 0b1001
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
                _ => unreachable!(),
            };

            (rhs << 10) | (src << 9) | (op << 7) | (reg_code << 4) | 0b0011
        }
        Instruction::LdrStr(r0, addr_reg, is_str, shift) => {
            let r0_code = (*r0 as u16) & 0b111;
            let addr_reg = (*addr_reg as u16) & 0b111;
            let shift = (*shift as u16) & 0b11111;
            let mut str = 0b0;
            if *is_str {
                str = 0b1;
            }

            (shift << 11) | (str << 10) | (addr_reg << 7) | (r0_code << 4) | 0b0100
        }
        Instruction::LdbStb(r0, addr_reg, is_str, shift) => {
            let r0_code = (*r0 as u16) & 0b111;
            let addr_reg = (*addr_reg as u16) & 0b111;
            let shift = (*shift as u16) & 0b11111;
            let mut str = 0b0;
            if *is_str {
                str = 0b1;
            }

            (shift << 11) | (str << 10) | (addr_reg << 7) | (r0_code << 4) | 0b1000
        }
        Instruction::Jmp(opt_reg, opt_imm) => {
            let (mode, value) = match (opt_reg, opt_imm) {
                (Some(reg), None) => (0b1, (*reg as u16) & 0b111),
                (None, Some(imm)) => (0b0, (*imm as u16) & 0b11111111111),
                _ => unreachable!(),
            };
            (value << 5) | (mode << 4) | 0b0110
        }
        Instruction::CondJmp(opt_reg, opt_imm) => {
            let (mode, value) = match (opt_reg, opt_imm) {
                (Some(reg), None) => (0b1, (*reg as u16) & 0b111),
                (None, Some(imm)) => (0b0, (*imm as u16) & 0b11111111111),
                _ => unreachable!(),
            };
            (value << 5) | (mode << 4) | 0b0101
        }
        Instruction::Cmp(reg, opt_reg, opt_imm, cmp_op) => {
            let r0 = (*reg as u16) & 0b111;
            let cmp = (*cmp_op as u16) & 0b111;
            let (mode, value) = match (opt_reg, opt_imm) {
                (Some(reg), None) => (0b1, (*reg as u16) & 0b111),
                (None, Some(imm)) => (0b0, (*imm as u16) & 0b11111),
                _ => unreachable!(),
            };
            (value << 11) | (mode << 10) | (cmp << 7) | (r0 << 4) | 0b0111
        }
        Instruction::CallRet(is_ret, addr) => {
            let imm = *addr & 0b11111111111;
            let flag = if *is_ret { 0b1 } else { 0b0 };

            (imm << 5) | ((flag & 0b1) << 4) | 0b1011
        },
    }
}

pub fn parse_assembly(code: &str) -> Result<Vec<Instruction>, AsmError> {
    let mut instructions = Vec::new();

    let empty_labels = HashMap::new();

    for (idx, line) in code.lines().enumerate() {
        println!("{} -> {}", idx * 2, line);

        let line = line.trim();
        if line.is_empty() || line.starts_with(";") {
            continue;
        }

        let inst = parse_assembly_line(line, &empty_labels)?;
        instructions.push(inst);
    }

    Ok(instructions)
}

// read the contents of the assembly file
pub fn resolve_and_parse_assembly(code: &str) -> Result<Vec<Instruction>, AsmError> {
    // labels should hold the address from the right next instruction to it
    let mut labels: HashMap<String, u16> = HashMap::new();
    let mut unresolved: HashMap<String, Vec<(usize, String)>> = HashMap::new();
    let mut instructions: Vec<Instruction> = vec![];

    // considering  we start at addr 0 and we increase 2 (given
    // that each instruction is 2 bytes long) we might be fine.
    let mut curr_inst_addr = 0;
    let mut line_number = 0;

    for line in code.lines().into_iter() {
        line_number += 1;
        let line = line.trim();

        if line.len() == 0 || line.starts_with(";") {
            // don't need to increase addr
            continue;
        }

        // found a label
        if line.ends_with(":") {
            let label = line.trim_end_matches(":");

            // need to figure out how to calculate
            // the instruction address offset
            labels.insert(label.to_string(), curr_inst_addr);

            if let Some(unresolved_label_loc) = unresolved.get(label) {
                for (inst_idx, inst_line) in unresolved_label_loc.iter() {
                    match parse_assembly_line(inst_line.as_ref(), &labels) {
                        Ok(inst) => instructions[*inst_idx] = inst,
                        Err(err) => eprintln!("while resolving label: {:?}", err),
                    }
                }

                unresolved.remove(label);
            }

            continue;
        }

        match parse_assembly_line(line.as_ref(), &labels) {
            Ok(inst) => instructions.push(inst),
            Err(AsmError::UnresolvedLabel(label)) => {
                instructions.push(Instruction::Noop);

                match unresolved.entry(label) {
                    Entry::Occupied(mut entry) => {
                        entry
                            .get_mut()
                            .push((instructions.len() - 1, line.to_string()));
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(vec![(instructions.len() - 1, line.to_string())]);
                    }
                };
            }
            Err(err) => {
                eprintln!("problems at line: {}: {}", line_number, line.to_string());
                return Err(err)
            },
        }

        curr_inst_addr += 2;
    }

    Ok(instructions)
}

pub fn encode_instructions(instructions: &[Instruction]) -> Vec<u16> {

    instructions.iter().map(encode_instruction).collect()
}

type ParserFn<'a> = Box<dyn Fn(&[&str]) -> Result<Instruction, AsmError> + 'a>;

pub fn parse_assembly_line<'a>(
    line: &str,
    labels: &'a HashMap<String, u16>,
) -> Result<Instruction, AsmError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Err(AsmError::InvalidFormat);
    }

    let instruction = parts[0].to_uppercase();
    let parser: ParserFn = match instruction.as_str() {
        "MOV" => Box::new(parse_mov),
        "MSL" => Box::new(parse_mov_shift(true)),
        "MSR" => Box::new(parse_mov_shift(false)),

        "CPY" => Box::new(parse_copy),
        "ADD" => Box::new(parse_arithmetic(ArithmeticOp::Add)),
        "SUB" => Box::new(parse_arithmetic(ArithmeticOp::Sub)),
        "MUL" => Box::new(parse_arithmetic(ArithmeticOp::Mul)),
        "DIV" => Box::new(parse_arithmetic(ArithmeticOp::Div)),
        "LDR" => Box::new(parse_ldr_str(false, false)),
        "STR" => Box::new(parse_ldr_str(false, true)),
        "LDB" => Box::new(parse_ldr_str(true, false)),
        "STB" => Box::new(parse_ldr_str(true, true)),
        "JMP" => Box::new(parse_jmp(false, labels)),
        "CJP" => Box::new(parse_jmp(true, labels)),
        "EQ" => Box::new(parse_comparision(CompareOp::Eq)),
        "NEQ" => Box::new(parse_comparision(CompareOp::NotEq)),
        "LT" => Box::new(parse_comparision(CompareOp::Less)),
        "LTE" => Box::new(parse_comparision(CompareOp::LessEq)),
        "GT" => Box::new(parse_comparision(CompareOp::Greater)),
        "GTE" => Box::new(parse_comparision(CompareOp::GreaterEq)),

        "RET" => Box::new(parse_ret),
        "CALL" => Box::new(parse_call(labels)),
        _ => return Err(AsmError::InvalidInstruction),
    };

    parser(&parts[1..])
}

fn parse_comparision(cmp_op: CompareOp) -> impl Fn(&[&str]) -> Result<Instruction, AsmError> {
    move |args: &[&str]| -> Result<Instruction, AsmError> {
        if args.len() != 2 {
            return Err(AsmError::InvalidInstruction);
        }

        let r0 = args[0].trim_end_matches(",").parse::<Register>()?;

        let (opt_reg, opt_imm) = if args[1].starts_with("#") {
            (None, Some(parse_immediate(args[1])?))
        } else {
            (Some(args[1].parse::<Register>()?), None)
        };

        Ok(Instruction::Cmp(r0, opt_reg, opt_imm, cmp_op))
    }
}

fn parse_jmp<'a>(
    cond: bool,
    labels: &'a HashMap<String, u16>,
) -> impl Fn(&[&str]) -> Result<Instruction, AsmError> {
    move |args: &[&str]| -> Result<Instruction, AsmError> {
        if args.len() != 1 {
            return Err(AsmError::InvalidInstruction);
        }

        let (opt_reg, opt_imm) = if args[0].starts_with("#") {
            (None, Some(parse_immediate(args[0])?))
        } else {
            match args[0].parse::<Register>() {
                Ok(reg) => (Some(reg), None),
                Err(_) => {
                    if let Some(jmp_addr) = labels.get(args[0]) {
                        (None, Some(jmp_addr.clone()))
                    } else {
                        return Err(AsmError::UnresolvedLabel(args[0].to_string()));
                    }
                }
            }
        };

        if cond {
            return Ok(Instruction::CondJmp(opt_reg, opt_imm));
        }

        Ok(Instruction::Jmp(opt_reg, opt_imm))
    }
}

fn parse_call<'a>(labels: &'a HashMap<String, u16>) -> impl Fn(&[&str]) -> Result<Instruction, AsmError>  {
    move |args: &[&str]| -> Result<Instruction, AsmError> {
        if args.len() != 1 {
            return Err(AsmError::InvalidInstruction);
        }

        let addr = if args[0].starts_with("#") {
            parse_immediate(args[0])?
        } else {
            if let Some(jmp_addr) = labels.get(args[0]) {
                *jmp_addr
            } else {
                return Err(AsmError::UnresolvedLabel(args[0].to_string()));
            }
        };

        return Ok(Instruction::CallRet(false, addr));
    }   
}


// LDR A, [B, #4]
fn parse_ldr_str(is_byte: bool, is_str: bool) -> impl Fn(&[&str]) -> Result<Instruction, AsmError> {
    move |args: &[&str]| -> Result<Instruction, AsmError> {
        if args.len() > 3 {
            return Err(AsmError::InvalidInstruction);
        }

        let reg_dst = args[0].trim_end_matches(',').parse::<Register>()?;
        if !args[1].starts_with('[') {
            let reg_src = args[1].parse::<Register>()?;
            if is_byte {
                return Ok(Instruction::LdbStb(reg_dst, reg_src, is_str, 0));
            } else {
                return Ok(Instruction::LdrStr(reg_dst, reg_src, is_str, 0));
            }
        }

        let reg_src = args[1].trim_start_matches('[').parse::<Register>()?;
        let shift = parse_immediate(args[2].trim_end_matches(']'))?;

        if is_byte {
            Ok(Instruction::LdbStb(
                reg_dst,
                reg_src,
                is_str,
                shift.try_into().map_err(|_| AsmError::InvalidImmediate)?,
            ))
        } else {
            Ok(Instruction::LdrStr(
                reg_dst,
                reg_src,
                is_str,
                shift.try_into().map_err(|_| AsmError::InvalidImmediate)?,
            ))
        }
    }
}

fn parse_arithmetic(op: ArithmeticOp) -> impl Fn(&[&str]) -> Result<Instruction, AsmError> {
    move |args: &[&str]| -> Result<Instruction, AsmError> {
        if args.len() > 2 {
            return Err(AsmError::InvalidInstruction);
        }

        let reg = args[0].trim_end_matches(',').parse::<Register>()?;
        if args[1].starts_with('#') {
            Ok(Instruction::Arith(
                reg,
                None,
                Some(parse_immediate(args[1])?),
                op,
            ))
        } else {
            Ok(Instruction::Arith(
                reg,
                Some(args[1].parse::<Register>()?),
                None,
                op,
            ))
        }
    }
}

fn parse_immediate(s: &str) -> Result<u16, AsmError> {
    if s.starts_with("#") {
        let value = &s[1..];

        if value.starts_with("0x") {
            let value = &value[2..];
            dbg!();
            return u16::from_str_radix(value, 16).map_err(|_| AsmError::InvalidImmediate);
        } else {
            return value.parse::<u16>().map_err(|_| AsmError::InvalidImmediate);
        }
    }

    Err(AsmError::InvalidImmediate)
}

fn parse_copy(args: &[&str]) -> Result<Instruction, AsmError> {
    if args.len() != 2 {
        return Err(AsmError::InvalidInstruction);
    }

    let src_reg = args[0].trim_end_matches(',').parse::<Register>()?;
    let dst_reg = args[1].parse::<Register>()?;

    Ok(Instruction::Cpy(src_reg, dst_reg))
}

fn parse_ret(_args: &[&str]) -> Result<Instruction, AsmError> {
    return Ok(Instruction::CallRet(true, 0));
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

        Ok(Instruction::MovShift(
            reg,
            shift.try_into().map_err(|_| AsmError::InvalidInstruction)?,
            dir,
            value,
        ))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::machine::{ArithmeticOp, CompareOp, Instruction, Register};

    use super::{encode_instruction, parse_assembly_line};

    #[test]
    fn test_encode_instruction() {
        let mov = Instruction::Mov(Register::C, 10);
        let inst = encode_instruction(&mov);
        assert_eq!(0b0000010100100001, inst);

        let msl = Instruction::MovShift(Register::B, 4, true, 4);
        let inst = encode_instruction(&msl);
        assert_eq!(0b0010011000010010, inst);

        let msr = Instruction::MovShift(Register::B, 4, false, 4);
        let inst = encode_instruction(&msr);
        assert_eq!(0b0010001000010010, inst);

        let add = Instruction::Arith(Register::B, Some(Register::C), None, ArithmeticOp::Add);
        let inst = encode_instruction(&add);
        assert_eq!(0b0000101000010011, inst);

        let add_imm = Instruction::Arith(Register::C, None, Some(8), ArithmeticOp::Div);
        let inst = encode_instruction(&add_imm);
        assert_eq!(0b0010000110100011, inst);

        let str = Instruction::LdrStr(Register::B, Register::SP, true, 0);
        let inst = encode_instruction(&str);
        assert_eq!(0b0000011000010100, inst);

        let ldr = Instruction::LdrStr(Register::B, Register::A, false, 10);
        let inst = encode_instruction(&ldr);
        assert_eq!(0b0101000000010100, inst);

        let call = Instruction::CallRet(false, 10);
        let inst = encode_instruction(&call);
        assert_eq!(0b0000000101001011, inst);

        let ret = Instruction::CallRet(true, 0);
        let inst = encode_instruction(&ret);
        assert_eq!(0b0000000000011011, inst);
    }

    #[test]
    fn test_assembly_line() {
        let empty = HashMap::new();

        let input = "MOV A, #10";
        let inst = parse_assembly_line(input, &empty).unwrap();
        assert_eq!(inst, Instruction::Mov(Register::A, 10));

        let input = "MSL A, [#11 #4]";
        let inst = parse_assembly_line(input, &empty).unwrap();
        assert_eq!(inst, Instruction::MovShift(Register::A, 4, true, 11));

        let input = "MSR B, [#15 #2]";
        let inst = parse_assembly_line(input, &empty).unwrap();
        assert_eq!(inst, Instruction::MovShift(Register::B, 2, false, 15));

        let input = "ADD A, #10";
        let inst = parse_assembly_line(input, &empty).unwrap();
        assert_eq!(
            inst,
            Instruction::Arith(Register::A, None, Some(10), ArithmeticOp::Add)
        );

        let input = "SUB B, SP";
        let inst = parse_assembly_line(input, &empty).unwrap();
        assert_eq!(
            inst,
            Instruction::Arith(Register::B, Some(Register::SP), None, ArithmeticOp::Sub)
        );

        let input = "MUL C, #5";
        let inst = parse_assembly_line(input, &empty).unwrap();
        assert_eq!(
            inst,
            Instruction::Arith(Register::C, None, Some(5), ArithmeticOp::Mul)
        );

        let input = "MUL A, B";
        let inst = parse_assembly_line(input, &empty).unwrap();
        assert_eq!(
            inst,
            Instruction::Arith(Register::A, Some(Register::B), None, ArithmeticOp::Mul)
        );

        let input = "DIV M, #16";
        let inst = parse_assembly_line(input, &empty).unwrap();
        assert_eq!(
            inst,
            Instruction::Arith(Register::M, None, Some(16), ArithmeticOp::Div)
        );

        let input = "DIV BP, A";
        let inst = parse_assembly_line(input, &empty).unwrap();
        assert_eq!(
            inst,
            Instruction::Arith(Register::BP, Some(Register::A), None, ArithmeticOp::Div)
        );

        let input = "STR SP, A";
        let inst = parse_assembly_line(input, &empty).unwrap();
        assert_eq!(
            inst,
            Instruction::LdrStr(Register::SP, Register::A, true, 0)
        );

        let input = "LDR C, [SP #4]";
        let inst = parse_assembly_line(input, &empty).unwrap();
        assert_eq!(
            inst,
            Instruction::LdrStr(Register::C, Register::SP, false, 4)
        );

        let input = "JMP #10";
        let inst = parse_assembly_line(input, &empty).unwrap();
        assert_eq!(inst, Instruction::Jmp(None, Some(10)));

        let input = "JMP A";
        let inst = parse_assembly_line(input, &empty).unwrap();
        assert_eq!(inst, Instruction::Jmp(Some(Register::A), None));

        let input = "CJP #10";
        let inst = parse_assembly_line(input, &empty).unwrap();
        assert_eq!(inst, Instruction::CondJmp(None, Some(10)));

        let input = "EQ A, B";
        let inst = parse_assembly_line(input, &empty).unwrap();
        assert_eq!(
            inst,
            Instruction::Cmp(Register::A, Some(Register::B), None, CompareOp::Eq)
        );

        let input = "LT A, #10";
        let inst = parse_assembly_line(input, &empty).unwrap();
        assert_eq!(
            inst,
            Instruction::Cmp(Register::A, None, Some(10), CompareOp::Less)
        );

        let input = "ADD FLAGS, #1";
        let inst = parse_assembly_line(input, &empty).unwrap();
        assert_eq!(
            inst,
            Instruction::Arith(Register::FLAGS, None, Some(1), ArithmeticOp::Add)
        );

        let input = "CALL #1024";
        let inst = parse_assembly_line(input, &empty).unwrap();
        assert_eq!(
            inst,
            Instruction::CallRet(false, 1024)
        );

        let input = "RET";
        let inst = parse_assembly_line(input, &empty).unwrap();
        assert_eq!(
            inst,
            Instruction::CallRet(true, 0)
        );
    }
}
