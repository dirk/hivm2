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
trait HostReadWriteExt: io::Read + ReadBytesExt {
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
impl<R: io::Read + ReadBytesExt> HostReadWriteExt for R {}

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

fn read_args(input: &mut Cursor<BBytes>) -> Vec<u8> {
    let num_args          = input.read_hu8();
    let mut args: Vec<u8> = vec![];

    for _ in 0..num_args {
        args.push(input.read_hu8())
    }

    args
}

/// Binary format: `addr:u64 num_args:u8 [arg:u8] out:u8`
///
/// **Note**: There will be 0 or more `arg` items corresponding to `num_args`.
impl BinarySerializable for BCall {
    fn from_binary(input: &mut Cursor<BBytes>) -> BCall {
        let addr = input.read_hu64();
        let args = read_args(input);
        let out  = reg_to_option(input.read_hu8());

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
        let args = read_args(input);
        let out  = reg_to_option(input.read_hu8());

        BCallNative { id: id, args: args, out: out, }
    }
}

pub struct BReturn {
    arg: Option<Reg>,
}

impl BinarySerializable for BReturn {
    fn from_binary(input: &mut Cursor<BBytes>) -> BReturn {
        let arg = reg_to_option(input.read_hu8());
        BReturn { arg: arg, }
    }
}

pub struct BSetLocal {
    idx: Local,
    arg: Reg,
}

impl BinarySerializable for BSetLocal {
    fn from_binary(input: &mut Cursor<BBytes>) -> BSetLocal {
        let idx = input.read_hu16();
        let arg = input.read_hu8();
        BSetLocal { idx: idx, arg: arg, }
    }
}

pub struct BGetLocal {
    idx: Local,
    out: Reg,
}

pub struct BEntry {
    /// Number of local variable slots
    num_locals: u16,
}

pub struct BGetArg {
    /// Index of the argument, pass 255 to get the total number of arguments passed
    idx: u8,
    out: Reg,
}

/// No-op entry to a function that sets up the local slots for the function. Must always be first
/// op in a function;
pub struct BFnEntry {
    /// Defines the number of local slots
    locals: u16,
}
