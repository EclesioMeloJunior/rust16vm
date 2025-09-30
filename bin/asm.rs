use rust16vm::{
    asm::{encode_instruction, resolve_and_parse_assembly},
    machine::Instruction,
};
#[allow(dead_code)]
// asm [output] [input files...]

// asm file.s file.bin -> outputs the encoded instructions
// asm file.s file.S -> will resolve the labels from .s file
// asm file.S file.bin -> outputs the encoded instructions
use std::env;
use std::{
    fs::File,
    io::{BufReader, Read, Write},
    path::Path,
};

fn main() -> Result<(), ()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("expected 2 positional args, received {}", args.len() - 1);
        return Err(());
    }

    let output_file = match Extension::try_from(args[1].to_string()) {
        Ok(ext) => ext,
        Err(ext_err) => {
            eprintln!("checking output file {} extension: {}", args[1], ext_err);
            return Err(());
        }
    };

    for input in &args[2..] {
        let _input_file = match Extension::try_from(input.to_string()) {
            Ok(ext) => ext,
            Err(ext_err) => {
                eprintln!("checking input file {} extension: {}", input, ext_err);
                return Err(());
            }
        };

        // place this code into a speciallized method bind to each Extension variant
        let path = Path::new(input);
        let open_file = File::open(path);
        match open_file {
            Ok(file) => {
                let mut buf = BufReader::new(file);
                let mut asm_str = String::new();

                match buf.read_to_string(&mut asm_str) {
                    Ok(_) => {}
                    Err(buf_err) => {
                        eprintln!("while reading input file {}: {}", input, buf_err);
                        return Err(());
                    }
                }

                let instructions = match resolve_and_parse_assembly(
                    asm_str.as_ref(),
                ) {
                    Ok(instructions) => instructions,
                    Err(err) => {
                        eprintln!("resolving input file {}: {:?}", input, err);
                        return Err(());
                    }
                };

                match output_file.write_instructions(instructions) {
                    Err(err) => {
                        eprintln!("{}", err);
                        return Err(());
                    }
                    Ok(()) => {
                        println!("generated {} file", args[1]);
                    }
                }
            }
            Err(io_err) => {
                eprintln!("opening {}: {}", input, io_err);
                return Err(());
            }
        }
    }

    Ok(())
}

enum Extension {
    BinaryExt(String),
    UnresolvedTextExt(String),
    ResolvedTextExt(String),
}

impl Extension {
    pub fn write_instructions(&self, instructions: Vec<Instruction>) -> Result<(), String> {
        match self {
            Extension::BinaryExt(f) => {
                let encoded: Vec<u16> = instructions
                    .iter()
                    .map(|inst| encode_instruction(inst))
                    .collect();

                let bin_instructions: Vec<u8> =
                    encoded.iter().flat_map(|inst| inst.to_le_bytes()).collect();

                let mut file = match File::create(f) {
                    Ok(bin_file) => bin_file,
                    Err(err) => return Err(format!("creating binary file: {}", err)),
                };

                match file.write_all(&bin_instructions) {
                    Ok(()) => Ok(()),
                    Err(err) => Err(format!("writing to binary file: {}", err)),
                }
            }
            _ => Err(String::from("not supported")),
        }
    }
}

impl TryFrom<String> for Extension {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Path::new(&value.clone())
            .extension()
            .and_then(|ext| ext.to_str())
            .map_or(
                Err(String::from("could not get file extension")),
                |ext_str| match ext_str {
                    "bin" => Ok(Extension::BinaryExt(value)),
                    "S" => Ok(Extension::ResolvedTextExt(value)),
                    "s" => Ok(Extension::UnresolvedTextExt(value)),
                    _ => Err(format!("unsupported extension: {}", ext_str)),
                },
            )
    }
}
