# rust16vm

A 16bit virtual machine with 8 registers.

- Current instructions:
  - MOV
  - ADD/SUB/MUL/DIV
  - STR
  - LDR
  - JMP
  - EQ/NEQ/LT(E)/GT(E)

## Specs

The VM uses 16bit wide instructions, loaded previusly at the memory, each instruction occupies 2 bytes in memory. It executes a naive fetch/decode/executes loop.

Register A, B and C are General Purpose registers.
Register M, SP, BP, PC, FLAGS are reserved.

### A simple for loop

Here is a small example on how the virtual machine can run an arbitrary set of instructions:
1. You can use the macro `rv16asm!` under the `src/asm` module to parse the instructions to binary that the VM can understand and execute
```
let program = rv16asm! {
  "MOV A, #0",

  "EQ A, #10",   // loop
  "CJP #10",
  "ADD A, #1",
  "JMP #2",      // jmp to instruction at addr 2

  "ADD FLAGS, #1" // halts machine
};

let mut mem = LinearMemory::new(1024);

for (idx, inst) in program.iter().enumerate() {
  assert!(mem.write2((idx * 2) as u16, *inst));
}

let mut machine = Machine::new(mem);
while let Ok(_) = machine.step() {
  machine.print_regs();
}

assert_eq!(machine.registers[Register::A as usize], 10);
```

### A complex for loop

I've made a more complex for loop that uses the terminal device to show
the numbers under iteration. The file combines char arithmetic, stack size and some sort  of function calling using the current instruction set.

If you want to try it out you need to 
1 - Use the assembler to generate the binary file: 
```
cargo build --release && ./target/release/asm output.bin ./testdata/loop.s
```
2 - The run the generated binary! 

```
./target/release/vm ./output.bin
```
