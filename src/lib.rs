extern crate byteorder;

#[macro_use]
extern crate nom;

pub mod asm;
pub mod asm_parser;
/// Compiles assembly to bytecode.
pub mod asm_compiler;
pub mod vm;
