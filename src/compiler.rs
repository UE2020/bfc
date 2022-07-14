use dynasmrt::{
    dynasm,
    x64::{Assembler, X64Relocation},
    DynasmApi, DynasmLabelApi,
};

use super::{Instruction, MEMORY_SIZE};
use std::io::Read;

macro_rules! asm {
    ($ops:ident $($t:tt)*) => {
        {
            dynasm!($ops
                ; .arch x64
                ; .alias idx, rdx
                ; .alias retval, rax
                $($t)*
            )
        }
    }
}

macro_rules! prologue {
    ($ops:ident) => {{
        let start = $ops.offset();
        asm!($ops
            ; sub rsp, 0x28
            ; mov [rsp + 0x30], rcx
            ; mov [rsp + 0x40], r8
            ; mov [rsp + 0x48], r9
        );
        start
    }};
}

macro_rules! epilogue {
    ($ops:ident, $e:expr) => {
        asm!($ops
        ; mov retval, $e
        ; add rsp, 0x28
        ; ret
    );};
}

macro_rules! call_extern {
    ($ops:ident, $addr:expr) => {asm!($ops
        ; mov [rsp + 0x38], rdx
        ; mov rax, QWORD $addr as _
        ; call rax
        ; mov rcx, [rsp + 0x30]
        ; mov rdx, [rsp + 0x38]
        ; mov r8,  [rsp + 0x40]
        ; mov r9,  [rsp + 0x48]
    );};
}

pub struct Program {
    code: dynasmrt::ExecutableBuffer,
    start: dynasmrt::AssemblyOffset,
}

impl Program {
    pub fn compile_program(program: &[Instruction], ops: &mut Assembler) {
        for instruction in program {
            match instruction {
                &Instruction::IncrementPtr(i) => asm!(ops
                    ; add idx, i as i32
                ),
                &Instruction::DecrementPtr(i) => asm!(ops
                    ; sub idx, i as i32
                ),
                &Instruction::Increment(i) => asm!(ops
                    ; add BYTE [idx], i as i8
                ),
                &Instruction::Decrement(i) => asm!(ops
                    ; sub BYTE [idx], i as i8
                ),
                &Instruction::PrintData => asm!(ops
                    ;; call_extern!(ops, State::putchar)
                ),
                &Instruction::ReadStdin => asm!(ops
                    ;; call_extern!(ops, State::getchar)
                ),
                &Instruction::Loop(ref body) => {
                    if body.as_slice() == &[Instruction::Decrement(1)] {
                        asm!(ops
                            ; mov BYTE [idx], 0
                        );
                    } else if body.as_slice() == &[Instruction::DecrementPtr(1)] {
                        asm!(ops
                            ; mov idx, 0
                        );
                    } else {
                        let begin_loop = ops.new_dynamic_label();
                        let end_loop = ops.new_dynamic_label();
                        asm!(ops
                            ; =>begin_loop
                            ; cmp BYTE [idx], 0
                            ; jz =>end_loop
                        );
                        Self::compile_program(body, ops);
                        asm!(ops
                            ; jmp =>begin_loop
                            ; =>end_loop
                        );
                    }
                }
                _ => {}
            }
        }
    }

    pub fn compile(program: &[Instruction]) -> Program {
        let mut ops = dynasmrt::x64::Assembler::new().unwrap();
        let start = prologue!(ops);

        Self::compile_program(program, &mut ops);

        asm!(ops
            ;; epilogue!(ops, 0)
        );

        let code = ops.finalize().unwrap();
        Program { code, start }
    }

    pub fn run(self, state: &mut State) -> Result<(), &'static str> {
        let f: extern "win64" fn(*mut State, *mut u8, *mut u8, *const u8) -> u8 =
            unsafe { std::mem::transmute(self.code.ptr(self.start)) };
        let start = state.data.as_mut_ptr();
        let end = unsafe { start.offset(MEMORY_SIZE as isize) };
        let res = f(state, start, start, end);
        Ok(())
    }
}

pub struct State {
    pub data: [u8; MEMORY_SIZE],
}

impl State {
    unsafe extern "win64" fn putchar(state: *mut State, cell: *mut u8) {
        print!("{}", *cell as char);
    }

    unsafe extern "win64" fn getchar(state: *mut State, cell: *mut u8) {
        std::io::stdin()
            .read_exact(std::slice::from_raw_parts_mut(cell, 1))
            .expect("failed to read stdin");
    }
}
