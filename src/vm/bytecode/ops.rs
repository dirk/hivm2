#![allow(dead_code)]

use byteorder::{ByteOrder, NativeEndian, ReadBytesExt};
use std::io;
use std::io::Cursor;

type BBytes<'a> = &'a [u8];

trait BinarySerializable {
    fn from_binary(&mut Cursor<BBytes>) -> Self;

    // fn write_binary(&self, &mut [u8]);
}

/// Returns Some(Reg) if the given register number is not the null register (255).
fn reg_to_option(reg: Reg) -> Option<Reg> {
    if reg == 255 {
        None
    } else {
        Some(reg)
    }
}

/// Adds host-native read and write methods, ie. `read_hu64` ("read host unsigned 64").
trait NativeEndianReadWriteExt: io::Read + ReadBytesExt {
    fn read_hu8(&mut self) -> u8 {
        self.read_u8().unwrap()
    }

    fn read_hu16(&mut self) -> u16 {
        self.read_u16::<NativeEndian>().unwrap()
    }

    fn read_hu32(&mut self) -> u32 {
        self.read_u32::<NativeEndian>().unwrap()
    }

    fn read_hu64(&mut self) -> u64 {
        self.read_u64::<NativeEndian>().unwrap()
    }
}
// Add HostReadWriteExt to all readables
impl<R: io::Read + ReadBytesExt> NativeEndianReadWriteExt for R {}

trait ReadWriteTypesExt: NativeEndianReadWriteExt {
    fn read_reg(&mut self) -> u8    { self.read_hu8() }
    fn read_addr(&mut self) -> u64  { self.read_hu64() }
    fn read_local(&mut self) -> u16 { self.read_hu16() }

    fn read_args(&mut self) -> Vec<u8> {
        let num_args          = self.read_hu8();
        let mut args: Vec<u8> = vec![];

        for _ in 0..num_args {
            args.push(self.read_reg())
        }

        args
    }
}
impl<R: NativeEndianReadWriteExt> ReadWriteTypesExt for R {}

/// Register index
type Reg = u8;

/// Address of a function
type Addr = u64;

/// Index of a local variable slot
type Local = u16;

/// Call a function at a specific address in the virtual machine.
pub struct BCall {
    /// Address of the function to be called
    addr: Addr,
    /// Argument registers of the call
    args: Vec<Reg>,
    /// Output register for the return of the call (255 for null)
    out: Option<Reg>,
}

/// Binary format: `addr:u64 num_args:u8 [arg:u8] out:u8`
///
/// **Note**: There will be 0 or more `arg` items corresponding to `num_args`.
impl BinarySerializable for BCall {
    fn from_binary(input: &mut Cursor<BBytes>) -> BCall {
        let addr = input.read_addr();
        let args = input.read_args();
        let out  = reg_to_option(input.read_reg());

        BCall { addr: addr, args: args, out: out, }
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
}

pub struct BReturn {
    arg: Option<Reg>,
}

impl BinarySerializable for BReturn {
    fn from_binary(input: &mut Cursor<BBytes>) -> BReturn {
        let arg = reg_to_option(input.read_reg());
        BReturn { arg: arg, }
    }
}

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
