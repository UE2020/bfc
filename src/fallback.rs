use super::{Instruction, MEMORY_SIZE};
use std::io::Read;

pub struct Environment {
    pub data: [u8; MEMORY_SIZE],
    pub idx: usize,
}
pub type EvalAst = Box<dyn Fn(&mut Environment)>;

pub fn eval(inst: Instruction) -> EvalAst {
    match inst {
        Instruction::IncrementPtr(i) => Box::new(move |e: &mut Environment| e.idx += i),
        Instruction::DecrementPtr(i) => Box::new(move |e: &mut Environment| e.idx -= i),
        Instruction::Increment(i) => Box::new(move |e: &mut Environment| unsafe {
            *e.data.get_unchecked_mut(e.idx) += i as u8;
        }),
        Instruction::Decrement(i) => Box::new(move |e: &mut Environment| unsafe {
            *e.data.get_unchecked_mut(e.idx) -= i as u8;
        }),
        Instruction::PrintData => Box::new(|e: &mut Environment| unsafe {
            print!("{}", *e.data.get_unchecked(e.idx) as char)
        }),
        Instruction::ReadStdin => Box::new(|e: &mut Environment| unsafe {
            let mut input: [u8; 1] = [0; 1];
            std::io::stdin()
                .read_exact(&mut input)
                .expect("failed to read stdin");
            *e.data.get_unchecked_mut(e.idx) = input[0];
        }),
        Instruction::Loop(body) => {
            // first of all, compile the body
            let calls = body.iter().map(|c| eval(c.clone())).collect::<Vec<_>>();

            Box::new(move |e: &mut Environment| unsafe {
                while *e.data.get_unchecked(e.idx) != 0 {
                    for call in calls.iter() {
                        call(e);
                    }
                }
            })
        }
        _ => unimplemented!(),
    }
}
