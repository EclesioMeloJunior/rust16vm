#[macro_export]
macro_rules! rv16asm {
    () => { Vec::<u16>::new() };

    ($inst:expr) => {{
        use super::{parse_assembly, encode_instructions};
        let instructions = parse_assembly($instr)
            .expect(&format!("failed to parse asm: {}", $instr));

        encode_instructions(&instructions)
    }};
        
    ($($inst:expr),* $(,)?) => {{
        use $crate::asm::{parse_assembly, encode_instructions};
        let code = concat!($($inst, "\n"),*);
        let instructions = parse_assembly(code)
            .expect(&format!("failed to parse asm:\n{}", code));

        encode_instructions(&instructions)
    }};
}
