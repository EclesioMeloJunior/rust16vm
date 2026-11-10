#[macro_export]
macro_rules! rv16asm {
    () => { Vec::<u16>::new() };

    // delegate to the multi-instruction arm: its `$crate::` paths also
    // resolve when the macro is used from outside this crate
    ($inst:expr) => { $crate::rv16asm!($inst,) };

    ($($inst:expr),* $(,)?) => {{
        use $crate::asm::{parse_assembly, encode_instructions};
        let code = concat!($($inst, "\n"),*);
        let instructions = parse_assembly(code)
            .expect(&format!("failed to parse asm:\n{}", code));

        encode_instructions(&instructions)
    }};
}
