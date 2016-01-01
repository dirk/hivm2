use asm;
use vm::bytecode::ops::*;

use std::io::Write;

type ByteVec = Vec<u8>;

trait Compile {
    fn compile(&self) -> ByteVec;
}

impl Compile for asm::Fn {
    fn compile(&self) -> ByteVec {
        let mut bytes = vec![];

        let entry = BFnEntry { num_locals: 0, };
        bytes.write(&entry.to_binary()).unwrap();

        bytes
    }
}
