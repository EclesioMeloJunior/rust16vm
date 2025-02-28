use super::memory::Addressable;

#[allow(dead_code)]
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

    pub fn step(&mut self) -> Result<(), &'static str> {
        let pc = self.registers[Register::PC as usize];
        let inst = self.memory.read2(pc).unwrap();
        println!("{} @ {}", inst, pc);
        Ok(())
    }
}
