#![allow(dead_code)]

use super::types::*;
use super::util::*;

use std::io::{Cursor, Write};

pub type BBytes<'a> = &'a Vec<u8>;

/// Defines interface for reading and writing a ops (instructions) to/from bytecode. All ops in
/// this module must implement this trait so that the VM can decode its instruction sequence
/// well-known op structures.
pub trait BinarySerializable {
    fn from_binary(&mut Cursor<BBytes>) -> Self;
    fn to_binary(&self) -> Vec<u8>;
}

pub trait IntoOpConvertable {
    fn into_op(self) -> BOp;
}

#[derive(Clone, Debug)]
pub enum BOp {
    FnEntry(BFnEntry),
    GetLocal(BGetLocal),
    SetLocal(BSetLocal),
    Call(BCall),
    Invoke(BInvoke),
    PushAddress(BPushAddress),
    BranchIf(BBranchIf),
    BranchIfNot(BBranchIfNot),
    Return,
    Pop,
    Noop,
}
impl BOp {
    pub fn to_binary(self) -> Vec<u8> {
        let mut bytes = vec![self.opcode()];

        match self {
            BOp::FnEntry(e)     => bytes.write(&e.to_binary()).unwrap(),
            BOp::GetLocal(g)    => bytes.write(&g.to_binary()).unwrap(),
            BOp::SetLocal(s)    => bytes.write(&s.to_binary()).unwrap(),
            BOp::Call(c)        => bytes.write(&c.to_binary()).unwrap(),
            BOp::Invoke(i)      => bytes.write(&i.to_binary()).unwrap(),
            BOp::PushAddress(a) => bytes.write(&a.to_binary()).unwrap(),
            BOp::BranchIf(b)    => bytes.write(&b.to_binary()).unwrap(),
            BOp::BranchIfNot(b) => bytes.write(&b.to_binary()).unwrap(),
            BOp::Return         => 0,
            BOp::Pop            => 0,
            BOp::Noop           => 0,
        };

        bytes
    }

    /// Take a vector of ops and convert them to a binary op sequence.
    pub fn compile_ops(ops: Vec<BOp>) -> Vec<u8> {
        ops.into_iter().flat_map(|op| op.to_binary()).collect()
    }

    pub fn opcode(&self) -> u8 {
        match self {
            &BOp::FnEntry(_)     => 0,
            &BOp::GetLocal(_)    => 1,
            &BOp::SetLocal(_)    => 2,
            &BOp::Call(_)        => 3,
            &BOp::Invoke(_)      => 4,
            &BOp::Return         => 5,
            &BOp::PushAddress(_) => 6,
            &BOp::BranchIf(_)    => 7,
            &BOp::BranchIfNot(_) => 8,
            &BOp::Pop            => 9,
            &BOp::Noop           => 10,
        }
    }

    pub fn from_binary(input: &mut Cursor<BBytes>) -> Self {
        let op = input.read_hu8();

        match op {
            0  => BOp::FnEntry(BFnEntry::from_binary(input)),
            1  => BOp::GetLocal(BGetLocal::from_binary(input)),
            2  => BOp::SetLocal(BSetLocal::from_binary(input)),
            3  => BOp::Call(BCall::from_binary(input)),
            4  => BOp::Invoke(BInvoke::from_binary(input)),
            5  => BOp::Return,
            6  => BOp::PushAddress(BPushAddress::from_binary(input)),
            7  => BOp::BranchIf(BBranchIf::from_binary(input)),
            8  => BOp::BranchIfNot(BBranchIfNot::from_binary(input)),
            9  => BOp::Pop,
            10 => BOp::Noop,
            _  => panic!("Invalid opcode: {:?}"),
        }
    }

    /// Returns the offset of the given address field in the op's compiled bytecode
    #[allow(unused_variables)]
    pub fn addr_field_offset(&self, idx: u8) -> u64 {
        let offset = match self {
            &BOp::Call(_)        => 0,
            &BOp::BranchIf(_)    => 0,
            &BOp::BranchIfNot(_) => 0,
            _                    => panic!("Op has no address fields: {:?}", self),
        };

        // 1 byte needed for the actual opcode
        1 + offset
    }
}

/// Call a function at a specific address in the virtual machine.
#[derive(Clone, Debug)]
pub struct BCall {
    /// Address of the function to be called
    pub addr: Addr,
    /// Number of arguments that have been pushed to the stack.
    pub num_args: u8,
}
// addr:u64 num_args:u8
impl BinarySerializable for BCall {
    fn from_binary(input: &mut Cursor<BBytes>) -> BCall {
        let addr     = input.read_addr();
        let num_args = input.read_hu8();
        BCall { addr: addr, num_args: num_args, }
    }
    fn to_binary(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.write_addr(self.addr);
        bytes.write_hu8(self.num_args);
        bytes
    }
}
impl IntoOpConvertable for BCall {
    fn into_op(self) -> BOp {
        BOp::Call(self)
    }
}

/// Consume an address off the stack and call the function at that address.
#[derive(Clone, Debug)]
pub struct BInvoke {
    pub num_args: u8,
}
impl BinarySerializable for BInvoke {
    fn from_binary(input: &mut Cursor<BBytes>) -> BInvoke {
        let num_args = input.read_hu8();
        BInvoke { num_args: num_args, }
    }
    fn to_binary(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.write_hu8(self.num_args);
        bytes
    }
}
impl IntoOpConvertable for BInvoke {
    fn into_op(self) -> BOp {
        BOp::Invoke(self)
    }
}

/// Call a native function.
pub struct BCallNative {
    /// Internal identifier of the function
    id: u32,
    num_args: u8,
}

impl BinarySerializable for BCallNative {
    fn from_binary(input: &mut Cursor<BBytes>) -> BCallNative {
        let id       = input.read_hu32();
        let num_args = input.read_hu8();
        BCallNative { id: id, num_args: num_args, }
    }
    fn to_binary(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.write_hu32(self.id);
        bytes.write_hu8(self.num_args);
        bytes
    }
}

/// Return from a function.
// pub struct BReturn { }
//
// impl BinarySerializable for BReturn {
//     fn from_binary(input: &mut Cursor<BBytes>) -> BReturn {
//         BReturn { }
//     }
//     fn to_binary(&self) -> Vec<u8> {
//         vec![]
//     }
// }

/// Set the value of a local variable to that of the given argument.
#[derive(Clone, Debug)]
pub struct BSetLocal {
    pub idx: Local,
}
impl BinarySerializable for BSetLocal {
    fn from_binary(input: &mut Cursor<BBytes>) -> BSetLocal {
        let idx = input.read_local();
        BSetLocal { idx: idx, }
    }
    fn to_binary(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.write_local(self.idx);
        bytes
    }
}
impl IntoOpConvertable for BSetLocal {
    fn into_op(self) -> BOp {
        BOp::SetLocal(self)
    }
}

/// Get the value of a local variable.
#[derive(Clone, Debug)]
pub struct BGetLocal {
    pub idx: Local,
}
impl BinarySerializable for BGetLocal {
    fn from_binary(input: &mut Cursor<BBytes>) -> BGetLocal {
        let idx = input.read_local();
        BGetLocal { idx: idx, }
    }
    fn to_binary(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.write_local(self.idx);
        bytes
    }
}
impl IntoOpConvertable for BGetLocal {
    fn into_op(self) -> BOp {
        BOp::GetLocal(self)
    }
}

/// Get an argument from the stack frame of the current function.
pub struct BGetArg {
    /// Index of the argument, pass 255 to get the total number of arguments passed
    idx: u8,
}
impl BinarySerializable for BGetArg {
    fn from_binary(input: &mut Cursor<BBytes>) -> BGetArg {
        let idx = input.read_hu8();
        BGetArg { idx: idx, }
    }
    fn to_binary(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.write_hu8(self.idx);
        bytes
    }
}

/// No-op entry to a function that sets up the local slots for the function. Must always be first
/// op in a function.
#[derive(Clone, Debug)]
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
        bytes.write_hu16(self.num_locals);
        bytes
    }
}
impl IntoOpConvertable for BFnEntry {
    fn into_op(self) -> BOp {
        BOp::FnEntry(self)
    }
}

#[derive(Clone, Debug)]
pub struct BPushAddress {
    pub addr: Addr,
}
impl BinarySerializable for BPushAddress {
    fn from_binary(input: &mut Cursor<BBytes>) -> BPushAddress {
        let addr = input.read_addr();
        BPushAddress { addr: addr, }
    }
    fn to_binary(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.write_addr(self.addr);
        bytes
    }
}
impl IntoOpConvertable for BPushAddress {
    fn into_op(self) -> BOp {
        BOp::PushAddress(self)
    }
}

#[derive(Clone, Debug)]
pub struct BBranchIf {
    pub dest: Addr,
}
impl BinarySerializable for BBranchIf {
    fn from_binary(input: &mut Cursor<BBytes>) -> BBranchIf {
        let addr = input.read_addr();
        BBranchIf { dest: addr, }
    }
    fn to_binary(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.write_addr(self.dest);
        bytes
    }
}
impl IntoOpConvertable for BBranchIf {
    fn into_op(self) -> BOp {
        BOp::BranchIf(self)
    }
}

#[derive(Clone, Debug)]
pub struct BBranchIfNot {
    pub dest: Addr,
}
impl BinarySerializable for BBranchIfNot {
    fn from_binary(input: &mut Cursor<BBytes>) -> BBranchIfNot {
        let addr = input.read_addr();
        BBranchIfNot { dest: addr, }
    }
    fn to_binary(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.write_addr(self.dest);
        bytes
    }
}
impl IntoOpConvertable for BBranchIfNot {
    fn into_op(self) -> BOp {
        BOp::BranchIfNot(self)
    }
}
