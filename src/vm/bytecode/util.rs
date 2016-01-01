use super::types::*;

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use std::io;

/// Returns Some(Reg) if the given register number is not the null register (255).
pub fn reg_to_option(reg: Reg) -> Option<Reg> {
    if reg == 255 {
        None
    } else {
        Some(reg)
    }
}
/// Converts the option of a register to the bytecode integer types (0-254 for a register, 255
/// for null).
pub fn option_to_reg(reg: Option<Reg>) -> Reg {
    match reg {
        Some(r) => r,
        None    => 255
    }
}

/// Adds host-native read and write methods, ie. `read_hu64` ("read host unsigned 64").
pub trait NativeEndianReadExt: io::Read + ReadBytesExt {
    fn read_hu8(&mut self) -> u8   { self.read_u8().unwrap() }
    fn read_hu16(&mut self) -> u16 { self.read_u16::<NativeEndian>().unwrap() }
    fn read_hu32(&mut self) -> u32 { self.read_u32::<NativeEndian>().unwrap() }
    fn read_hu64(&mut self) -> u64 { self.read_u64::<NativeEndian>().unwrap() }
}
// Add HostReadWriteExt to all readables
impl<R: io::Read + ReadBytesExt> NativeEndianReadExt for R {}

/// Extension to `NativeEndianReadWriteExt` to add type-specific reading functions to work with
/// the correct size of the types in the bytecode.
pub trait ReadTypesExt: NativeEndianReadExt {
    fn read_reg(&mut self) -> u8    { self.read_hu8() }
    fn read_addr(&mut self) -> u64  { self.read_hu64() }
    fn read_local(&mut self) -> u16 { self.read_hu16() }

    fn read_args(&mut self) -> Vec<u8> {
        let num_args = self.read_hu8();

        (0..num_args).enumerate().map(|_| self.read_reg()).collect()
    }
}
impl<R: NativeEndianReadExt> ReadTypesExt for R {}

/// Extension to add type-specific writing functions for the various types in the bytecode.
pub trait WriteTypesExt {
    fn write_addr(&mut self, Addr);
    fn write_args(&mut self, Vec<u8>);
    fn write_local(&mut self, Local);
    fn write_reg(&mut self, Reg);
}
/// Enable writing bytecode types to `Vec<u8>`.
impl WriteTypesExt for Vec<u8> {
    fn write_addr(&mut self, addr: Addr) {
        self.write_u64::<NativeEndian>(addr).unwrap()
    }

    fn write_args(&mut self, args: Vec<u8>) {
        self.write_u8(args.len() as u8).unwrap();
        for a in args { self.write_reg(a) }
    }

    fn write_local(&mut self, local: Local) {
        self.write_u16::<NativeEndian>(local).unwrap()
    }

    fn write_reg(&mut self, reg: Reg) {
        self.write_u8(reg).unwrap()
    }
}

macro_rules! serialize {
    (
        $name:ident,
        from($from_arg:ident) $from_block:block,
        to($to_arg:ident) $to_block:expr
    ) => {
        impl BinarySerializable for $name {
            fn from_binary($from_arg: &mut Cursor<BBytes>) -> $name $from_block

            fn to_binary(&self) -> Vec<u8> {
                let $to_arg: Vec<u8> = vec![];
                $to_block;
                $to_arg
            }
        }
    };
}
