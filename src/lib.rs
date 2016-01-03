extern crate byteorder;

#[macro_use]
extern crate nom;

/// Syntax tree of the assembly.
pub mod asm;
/// Parses textual code into an assembly tree.
pub mod asm_parser;
/// Compiles assembly to bytecode.
pub mod asm_compiler;
/// The bytecode-interpreting stack virtual machine itself.
pub mod vm;
