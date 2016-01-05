use super::types::*;

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use std::io;

/// Adds host-native read and write methods, ie. `read_hu64` ("read host unsigned 64").
pub trait NativeEndianReadExt: io::Read + ReadBytesExt {
    fn read_hu8(&mut self) -> u8   { self.read_u8().unwrap() }
    fn read_hu16(&mut self) -> u16 { self.read_u16::<NativeEndian>().unwrap() }
    fn read_hu32(&mut self) -> u32 { self.read_u32::<NativeEndian>().unwrap() }
    fn read_hu64(&mut self) -> u64 { self.read_u64::<NativeEndian>().unwrap() }
}
impl<R: io::Read + ReadBytesExt> NativeEndianReadExt for R {}

pub trait NativeEndianWriteExt: io::Write + WriteBytesExt {
    fn write_hu8(&mut self, u: u8)   { self.write_u8(u).unwrap() }
    fn write_hu16(&mut self, u: u16) { self.write_u16::<NativeEndian>(u).unwrap() }
    fn write_hu32(&mut self, u: u32) { self.write_u32::<NativeEndian>(u).unwrap() }
    fn write_hu64(&mut self, u: u64) { self.write_u64::<NativeEndian>(u).unwrap() }
}
impl<R: io::Write + WriteBytesExt> NativeEndianWriteExt for R {}

/// Extension to `NativeEndianReadWriteExt` to add type-specific reading functions to work with
/// the correct size of the types in the bytecode.
pub trait ReadTypesExt: NativeEndianReadExt {
    fn read_addr(&mut self) -> u64  { self.read_hu64() }
    fn read_local(&mut self) -> u16 { self.read_hu16() }
}
impl<R: NativeEndianReadExt> ReadTypesExt for R {}

/// Extension to add type-specific writing functions for the various types in the bytecode.
pub trait WriteTypesExt {
    fn write_addr(&mut self, Addr);
    fn write_local(&mut self, Local);
}
/// Enable writing bytecode types to `Vec<u8>`.
impl WriteTypesExt for Vec<u8> {
    fn write_addr(&mut self, addr: Addr) {
        self.write_u64::<NativeEndian>(addr).unwrap()
    }

    fn write_local(&mut self, local: Local) {
        self.write_u16::<NativeEndian>(local).unwrap()
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
