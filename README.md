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
while let Ok(State::Continue) = machine.step()  {
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

### Instructions

#### MOV {destination_register}, #{immediate (9 bits)}
Moves the `immediate` value to the `destination_register`

```
MOV A, #10
```

#### MSL {destination_register}, [#{immediate (5 bits)}, #{shift_amount (3 bits)}]
Shifts the value of `destination_register` to the left by `shift_amount` and then perform an `OR` operation using the `immediate`

```
MSL B, [#1 #3]
```

#### MSR {destination_register}, [#{immediate (5 bits)}, #{shift_amount (3 bits)}]
Shifts the value of `destination_register` to the right by `shift_amount` and then perform an `OR` operation using the `immediate`
 
```
MSR B, [#1 #3]
```

#### ADD {destination_register}, {source_register}
Performs a arithmetic addition between `destination_register` and `source_register`. Stores the result in the `destination_register`

```
ADD A, B
```

#### ADD {destination_register}, #{immediate (6 bits)}
Performs a arithmetic addition between `destination_register` and `immediate`. Stores the result in the `destination_register`

```
ADD A, #10
```

#### SUB {destination_register}, {source_register}
Performs a arithmetic subtraction between `destination_register` and `source_register`. Stores the result in the `destination_register`

```
SUB A, B
```

#### SUB {destination_register}, #{immediate (6 bits)}
Performs a arithmetic subtraction between `destination_register` and `immediate`. Stores the result in the `destination_register`

```
 SUB A, #10
```

#### MUL {destination_register}, {source_register}
Performs a arithmetic multiplication between `destination_register` and `source_register`. Stores the result in the `destination_register`

```
MUL A, B
```

#### MUL {destination_register}, #{immediate (6 bits)}
Performs a arithmetic multiplication between `destination_register` and `immediate`. Stores the result in the `destination_register`

```
MUL A, #10
```

#### DIV {destination_register}, {source_register}
Performs a arithmetic division between `destination_register` and `source_register`. Stores the result in the `destination_register`

```
DIV A, B
```

#### DIV {destination_register}, #{immediate (6 bits)}
Performs a arithmetic division between `destination_register` and `immediate`. Stores the result in the `destination_register`

```
DIV A, #10
```

#### MOD (unstable)
To calculate the modulo (remainder of a division) of two numbers you should set the bit at index 1 of the FLAGS register to `1` and then perform the division instruction, the modulo will be placed in the stack

```
OR FLAGS, #2 //sets the bit at position 1 to be `1`
DIV A, B     // performs the division
LDR C, SP   // gets from the stack the module and store in C
```

#### EXPR {dst_register}, {base_reg}, {exponent_reg}
To calculate the exponentiation (Aⁿ).

```
MOV A, #2
MOV B, #3
; this will calculate 2 ^ 3
EXPR C, A, B
```
#### SQRT {dst_register}, {base_reg}, {exponent_reg}
To calculate the square root (√A).

```
MOV A, #4
; this calcutare √4
SQRTR C, A, #2
```

#### CPY {from_register} {to_register}
Copies a value from a memory address inside another memory address.

```
CPY A, B // where A and B holds memory addresses
```

#### LDR {from_register} {destination_register}
Loads a 2 bytes value from memory into `destination_register`

```
LDR A, SP // loads a value in the stack into the register A
```

#### STR {src_register} {addr_register}
Stores a 2 bytes value stored in `src_register` in the memory using the address from `addr_register`

```
  MOV A, #72
  MOV B, #0x0F
  STR A, B // stores the value 72 in the memory address 0x0F
```

#### LDB {from_register} {destination_register}
Loads a 1 byte value from memory into `destination_register`

```
LDB A, SP // loads a value in the stack into the register A
```

#### STB {src_register} {addr_register}
Stores a 1 bytes value stored in `src_register` in the memory using the address from `addr_register`


``` 
  MOV A, #72
  MOV B, #0x0F
  STB A, B // stores the value 72 in the memory address 0x0F
```

#### JMP {register}
Unconditional jump, changes the program counter register to be the value inside the given `register`

```
  MOV A, #0
  JMP A // change the VM program counter to read instruction at 0 address
```

#### JMP {immediate (11 bits)}
Unconditional jump, changes the program counter register to be the value inside the given `register`

```
JMP #0 // change the VM program counter to read instruction at 0 address
```

#### CJP {register}
Conditional jump, changes the program counter register to be the value inside the given `register` based on the FLAGS register bit at position 3, if 1 then it jumps otherwise proceed to the next instruction.

```
MOV B, #0
EQ A, #10
CJP B // if register A holds value 10 then it jumps to the location 0 (placed in register B)
```

#### CJP {immediate (11 bits)}
Conditional jump, changes the program counter register to be the value inside the given `register` based on the FLAGS register bit at position 3, if 1 then it jumps otherwise proceed to the next instruction.

```
EQ A, #10
CJP #0 // if register A holds value 10 then it jumps to the location 0
```

#### EQ {source_register}, {cmp_register}
Perfoms an equal comparision (==) between two registers, setting the FLAGS register bit at position 3 to `1` if true, `0` otherwise

```
EQ A, B
```

#### EQ {source_register}, {immediate (5 bits)}
Perfoms an equal comparision (==) between register and immediate, setting the FLAGS register bit at position 3 to `1` if true, `0` otherwise

```
EQ A, #100
```


#### NEQ {source_register}, {cmp_register}
Perfoms a not equal comparision (!=) between two registers, setting the FLAGS register bit at position 3 to `1` if true, `0` otherwise

```
NEQ A, B
```

#### NEQ {source_register}, {immediate (5 bits)}
Perfoms a not equal comparision (!=) between register and immediate, setting the FLAGS register bit at position 3 to `1` if true, `0` otherwise

```
NEQ A, #100
```

#### LT {source_register}, {cmp_register}
Perfoms a less than comparision (<) between two registers, setting the FLAGS register bit at position 3 to `1` if true, `0` otherwise

```
LT A, B
```

#### LT {source_register}, {immediate (5 bits)}
Perfoms a less than comparision (<) between register and immediate, setting the FLAGS register bit at position 3 to `1` if true, `0` otherwise

```
LT A, #100
```

#### LTE {source_register}, {cmp_register}
Perfoms a less than or equal comparision (<=) between two registers, setting the FLAGS register bit at position 3 to `1` if true, `0` otherwise

```
LTE A, B
```

#### LT {source_register}, {immediate (5 bits)}
Perfoms a less than or equal comparision (<=) between register and immediate, setting the FLAGS register bit at position 3 to `1` if true, `0` otherwise

```
LTE A, #100
```

#### GT {source_register}, {cmp_register}
Perfoms a greater than comparision (>) between two registers, setting the FLAGS register bit at position 3 to `1` if true, `0` otherwise

```
GT A, B
```

#### GT {source_register}, {immediate (5 bits)}
Perfoms a greater than comparision (>) between register and immediate, setting the FLAGS register bit at position 3 to `1` if true, `0` otherwise

```
GT A, #100
```

#### GTE {source_register}, {cmp_register}
Perfoms a greater than or equal comparision (>=) between two registers, setting the FLAGS register bit at position 3 to `1` if true, `0` otherwise

```
GTE A, B
```

#### GTE {source_register}, {immediate (5 bits)}
Perfoms a greater than or equal comparision (>=) between register and immediate, setting the FLAGS register bit at position 3 to `1` if true, `0` otherwise

```
GTE A, #100
```