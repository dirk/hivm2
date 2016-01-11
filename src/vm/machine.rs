use super::super::asm;
use super::super::asm_compiler::{CompiledRelocationTarget, CompileModule};
use super::bytecode::types::Addr;
use super::bytecode::util::NativeEndianWriteExt;

use std::collections::HashMap;
use std::fmt;
use std::io::Cursor;
use std::any::Any;
use std::mem;

pub type ValueBox<T: Any> = Box<T>;

/// Untyped pointer to a value
pub type ValuePointer = *mut usize;

pub trait IntoBox {
    unsafe fn into_box<T: Any + Sized>(self) -> ValueBox<T>;
}
impl IntoBox for ValuePointer {
    unsafe fn into_box<T: Any + Sized>(self) -> ValueBox<T> {
        Box::from_raw(self as *mut T)
    }
}

pub trait IntoPointer {
    unsafe fn into_value_pointer(self) -> ValuePointer;
}
impl<T: Any> IntoPointer for ValueBox<T> {
    unsafe fn into_value_pointer(self) -> ValuePointer {
        mem::transmute(self)
    }
}

pub type BoxedPrimitiveFn = Box<Fn(&mut Machine, &Frame)>;
pub struct PrimitiveFn(BoxedPrimitiveFn);

impl fmt::Debug for PrimitiveFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[native code]")
    }
}

pub type TableKey = String;

#[derive(Debug)]
pub enum TableValue {
    /// Pointer to the constant value in memory
    Const(ValuePointer),
    /// Pointer to the static value in memory
    Static(ValuePointer),
    /// Address in the machine's code for the function
    Defn(Addr),
    /// Primitive function
    Primitive(PrimitiveFn),
}

impl TableValue {
    fn as_addr(&self) -> u64 {
        match *self {
            TableValue::Defn(ptr) => ptr,
            _ => panic!("Cannot convert {:?} to Addr", self)
        }
    }

    pub fn with_fn(f: BoxedPrimitiveFn) -> Self {
        TableValue::Primitive(PrimitiveFn(f))
    }
}

pub struct SymbolTable {
    table: HashMap<TableKey, TableValue>,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            table: HashMap::new(),
        }
    }

    fn has_symbol(&self, symbol: &TableKey) -> bool {
        self.table.contains_key(symbol)
    }

    fn lookup_symbol(&self, symbol: &TableKey) -> &TableValue {
        self.table.get(symbol).unwrap()
    }

    pub fn set_symbol(&mut self, symbol: &TableKey, value: TableValue) {
        self.table.insert(symbol.clone(), value);
    }
}

/// The actual virtual machine
pub struct Machine {
    /// Bytecode stored in the virtual machine
    pub code: Vec<u8>,

    pub call_stack: Vec<Frame>,

    pub ip: Addr,

    pub stack: Vec<ValuePointer>,

    pub symbol_table: SymbolTable,
}

pub struct Frame {
    pub return_addr: Addr,
    pub args: Vec<ValuePointer>,
    pub slots: Vec<ValuePointer>,
}

trait ModuleLoad {
    fn load_module(&mut self, &mut asm::Module);
}

impl ModuleLoad for Machine {
    fn load_module(&mut self, module: &mut asm::Module) {
        use super::super::asm_compiler::CompiledRelocationTarget::*;

        let compiled        = module.compile();
        let ref relocations = compiled.relocations;

        let base_addr = self.code.len() as u64;
        self.code.extend(compiled.code);

        let mut writer = Cursor::new(&mut self.code[..]);

        for relocation in relocations {
            let module_addr = relocation.0;
            let final_addr  = base_addr + module_addr;
            writer.set_position(final_addr);

            let ref target: CompiledRelocationTarget = relocation.1;

            match target {
                &InternalAddress(target_module_addr) => {
                    let target_final_addr = base_addr + target_module_addr;
                    writer.write_hu64(target_final_addr);
                },
                &ExternalFunctionPath(ref path) => {
                    if self.symbol_table.has_symbol(path) {
                        let target = self.symbol_table.lookup_symbol(path);
                        writer.write_hu64(target.as_addr());
                    } else {
                        panic!("Symbol not found in symbol table: {:?}", path)
                    }
                },
            }
        }
    }// fn load_module
}
