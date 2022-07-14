use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

#[cfg(not(target_arch = "x86_64"))]
mod fallback;

#[cfg(target_arch = "x86_64")]
mod compiler;

pub const MEMORY_SIZE: usize = 16000; // 16kb

#[derive(Debug, StructOpt)]
#[structopt(name = "bfc", about = "A brainfuck engine with a JIT compiler (x86) and an optimized interpreter as a fallback.")]
struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    IncrementPtr(usize),
    DecrementPtr(usize),
    Increment(usize),
    Decrement(usize),
    PrintData,
    ReadStdin,
    Loop(Vec<Instruction>),
    Undefined,
}

fn parse(opcodes: Vec<char>) -> Vec<Instruction> {
    let mut program: Vec<Instruction> = Vec::new();
    let mut loop_stack = 0;
    let mut loop_start = 0;

    for (i, op) in opcodes.iter().enumerate() {
        if loop_stack == 0 {
            match op {
                '>' => {
                    let last = program.len() - 1;
                    if let Some(Instruction::IncrementPtr(ref mut x)) = program.get_mut(last) {
                        *x += 1;
                    } else {
                        program.push(Instruction::IncrementPtr(1));
                    }
                }
                '<' => {
                    let last = program.len() - 1;
                    if let Some(Instruction::DecrementPtr(ref mut x)) = program.get_mut(last) {
                        *x += 1;
                    } else {
                        program.push(Instruction::DecrementPtr(1));
                    }
                }
                '+' => {
                    let last = program.len() - 1;
                    if let Some(Instruction::Increment(ref mut x)) = program.get_mut(last) {
                        *x += 1;
                    } else {
                        program.push(Instruction::Increment(1));
                    }
                }
                '-' => {
                    let last = program.len() - 1;
                    if let Some(Instruction::Decrement(ref mut x)) = program.get_mut(last) {
                        *x += 1;
                    } else {
                        program.push(Instruction::Decrement(1));
                    }
                }
                '.' => program.push(Instruction::PrintData),
                ',' => program.push(Instruction::ReadStdin),

                '[' => {
                    loop_start = i;
                    loop_stack += 1;
                }

                ']' => panic!("loop ending at #{} has no beginning", i),

                _ => {}
            };
        } else {
            match op {
                '[' => {
                    loop_stack += 1;
                }
                ']' => {
                    loop_stack -= 1;

                    if loop_stack == 0 {
                        program.push(Instruction::Loop(parse(
                            opcodes[loop_start + 1..i].to_vec(),
                        )));
                    }
                }
                _ => (),
            }
        }
    }

    if loop_stack != 0 {
        panic!(
            "loop that starts at #{} has no matching ending!",
            loop_start
        );
    }

    program
}

fn main() {
    let opt = Opt::from_args();

    let contents =
        fs::read_to_string(opt.input).expect("Something went wrong reading the input file");

    let chars = contents.chars().collect::<Vec<_>>();
    let instructions = parse(chars);

    // interpreter fallback
    #[cfg(not(target_arch = "x86_64"))]
    {
        use fallback::*;

        let instructions = instructions
            .iter()
            .map(|i| eval(i.clone()))
            .collect::<Vec<_>>();

        let mut env = Environment {
            data: [0; MEMORY_SIZE],
            idx: 0,
        };
        for inst in instructions {
            inst(&mut env);
        }
    }

    #[cfg(target_arch = "x86_64")]
    {
        use compiler::*;

        let program = Program::compile(&instructions);
        let mut state = State {
            data: [0; MEMORY_SIZE],
        };
        program.run(&mut state).unwrap();
    }

    println!();
}
