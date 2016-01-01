use super::types::*;

use byteorder::{NativeEndian, ReadBytesExt};
use std::io;

/// Returns Some(Reg) if the given register number is not the null register (255).
pub fn reg_to_option(reg: Reg) -> Option<Reg> {
    if reg == 255 {
        None
    } else {
        Some(reg)
    }
}

/// Adds host-native read and write methods, ie. `read_hu64` ("read host unsigned 64").
pub trait NativeEndianReadWriteExt: io::Read + ReadBytesExt {
    fn read_hu8(&mut self) -> u8   { self.read_u8().unwrap() }
    fn read_hu16(&mut self) -> u16 { self.read_u16::<NativeEndian>().unwrap() }
    fn read_hu32(&mut self) -> u32 { self.read_u32::<NativeEndian>().unwrap() }
    fn read_hu64(&mut self) -> u64 { self.read_u64::<NativeEndian>().unwrap() }
}
// Add HostReadWriteExt to all readables
impl<R: io::Read + ReadBytesExt> NativeEndianReadWriteExt for R {}

/// Extension to `NativeEndianReadWriteExt` to add type-specific reading and writing functions to
/// work with the correct size of the types in the bytecode.
pub trait ReadWriteTypesExt: NativeEndianReadWriteExt {
    fn read_reg(&mut self) -> u8    { self.read_hu8() }
    fn read_addr(&mut self) -> u64  { self.read_hu64() }
    fn read_local(&mut self) -> u16 { self.read_hu16() }

    fn read_args(&mut self) -> Vec<u8> {
        let num_args = self.read_hu8();

        (0..num_args).enumerate().map(|_| self.read_reg()).collect()
    }
}
impl<R: NativeEndianReadWriteExt> ReadWriteTypesExt for R {}

#[macro_export]
macro_rules! serialize {
    (
        $name:ident,
        from($from_arg:ident) $from_block:block
    ) => {
        impl BinarySerializable for $name {
            fn from_binary($from_arg: &mut Cursor<BBytes>) -> $name $from_block
        }
    };
}
