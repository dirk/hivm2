#![allow(dead_code)]

use super::types::*;
use super::util::*;

use byteorder::{NativeEndian, WriteBytesExt};
use std::io::Cursor;

pub type BBytes<'a> = &'a [u8];

/// Defines interface for reading and writing a ops (instructions) to/from bytecode. All ops in
/// this module must implement this trait so that the VM can decode its instruction sequence
/// well-known op structures.
pub trait BinarySerializable {
    fn from_binary(&mut Cursor<BBytes>) -> Self;
    fn to_binary(&self) -> Vec<u8>;
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

// addr:u64 num_args:u8 [arg:u8]* out:u8
impl BinarySerializable for BCall {
    fn from_binary(input: &mut Cursor<BBytes>) -> BCall {
        let addr = input.read_addr();
        let args = input.read_args();
        let out  = reg_to_option(input.read_reg());
        BCall { addr: addr, args: args, out: out, }
    }
    fn to_binary(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.write_addr(self.addr);
        bytes.write_args(self.args.clone());
        bytes.write_reg(option_to_reg(self.out));
        bytes
    }
}

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
    fn to_binary(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.write_u32::<NativeEndian>(self.id).unwrap();
        bytes.write_args(self.args.clone());
        bytes.write_reg(option_to_reg(self.out));
        bytes
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
    fn to_binary(&self) -> Vec<u8> {
        vec![option_to_reg(self.arg)]
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
    fn to_binary(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.write_local(self.idx);
        bytes.write_reg(self.arg);
        bytes
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
    fn to_binary(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.write_local(self.idx);
        bytes.write_reg(self.out);
        bytes
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
    fn to_binary(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.write_u8(self.idx).unwrap();
        bytes.write_reg(self.out);
        bytes
    }
}

/// No-op entry to a function that sets up the local slots for the function. Must always be first
/// op in a function;
pub struct BFnEntry {
    /// Defines the number of local slots
    pub num_locals: u16,
}

impl BinarySerializable for BFnEntry {
    fn from_binary(input: &mut Cursor<BBytes>) -> BFnEntry {
        let num_locals = input.read_hu16();
        BFnEntry { num_locals: num_locals, }
    }
    fn to_binary(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.write_u16::<NativeEndian>(self.num_locals).unwrap();
        bytes
    }
}
