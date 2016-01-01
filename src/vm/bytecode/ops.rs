#![allow(dead_code)]

use super::types::*;
use super::util::*;

use std::io::Cursor;

pub type BBytes<'a> = &'a [u8];

/// Defines interface for reading and writing a ops (instructions) to/from bytecode. All ops in
/// this module must implement this trait so that the VM can decode its instruction sequence
/// well-known op structures.
pub trait BinarySerializable {
    fn from_binary(&mut Cursor<BBytes>) -> Self;

    // fn write_binary(&self, &mut [u8]);
}

/// Call a function at a specific address in the virtual machine.
pub struct BCall {
    /// Address of the function to be called
    addr: Addr,
    /// Argument registers of the call
    args: Vec<Reg>,
    /// Output register for the return of the call (255 for null)
    out: Option<Reg>,
}
serialize!(BCall,
    // addr:u64 num_args:u8 [arg:u8]* out:u8
    from(input) {
        let addr = input.read_addr();
        let args = input.read_args();
        let out  = reg_to_option(input.read_reg());
        BCall { addr: addr, args: args, out: out, }
    }
);

/// Call a native function.
pub struct BCallNative {
    /// Internal identifier of the function
    id: u32,
    args: Vec<Reg>,
    out: Option<Reg>,
}

impl BinarySerializable for BCallNative {
    fn from_binary(input: &mut Cursor<BBytes>) -> BCallNative {
        let id   = input.read_hu32();
        let args = input.read_args();
        let out  = reg_to_option(input.read_hu8());

        BCallNative { id: id, args: args, out: out, }
    }
}

/// Return from a function.
pub struct BReturn {
    arg: Option<Reg>,
}

impl BinarySerializable for BReturn {
    fn from_binary(input: &mut Cursor<BBytes>) -> BReturn {
        let arg = reg_to_option(input.read_reg());
        BReturn { arg: arg, }
    }
}

/// Set the value of a local variable to that of the given argument.
pub struct BSetLocal {
    idx: Local,
    arg: Reg,
}

impl BinarySerializable for BSetLocal {
    fn from_binary(input: &mut Cursor<BBytes>) -> BSetLocal {
        let idx = input.read_local();
        let arg = input.read_reg();
        BSetLocal { idx: idx, arg: arg, }
    }
}

/// Get the value of a local variable.
pub struct BGetLocal {
    idx: Local,
    out: Reg,
}

impl BinarySerializable for BGetLocal {
    fn from_binary(input: &mut Cursor<BBytes>) -> BGetLocal {
        let idx = input.read_local();
        let out = input.read_reg();
        BGetLocal { idx: idx, out: out, }
    }
}

/// Get an argument from the stack frame of the current function.
pub struct BGetArg {
    /// Index of the argument, pass 255 to get the total number of arguments passed
    idx: u8,
    out: Reg,
}

impl BinarySerializable for BGetArg {
    fn from_binary(input: &mut Cursor<BBytes>) -> BGetArg {
        let idx = input.read_hu8();
        let out = input.read_reg();
        BGetArg { idx: idx, out: out, }
    }
}

/// No-op entry to a function that sets up the local slots for the function. Must always be first
/// op in a function;
pub struct BFnEntry {
    /// Defines the number of local slots
    num_locals: u16,
}

impl BinarySerializable for BFnEntry {
    fn from_binary(input: &mut Cursor<BBytes>) -> BFnEntry {
        let num_locals = input.read_hu16();
        BFnEntry { num_locals: num_locals, }
    }
}
