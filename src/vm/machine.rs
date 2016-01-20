use asm_compiler::{CompiledRelocationTarget, CompiledModule};
use super::bytecode::types::Addr;
use super::bytecode::util::NativeEndianWriteExt;

use std::collections::HashMap;
use std::fmt;
use std::io::Cursor;
use std::any::Any;
use std::mem;

pub type ValueBox<T> = Box<T>;

/// Untyped pointer to a value
pub type ValuePointer = *mut usize;

/// Convert a thing into a typed `ValueBox`.
pub trait IntoBox {
    unsafe fn into_box<T: Any + Sized>(self) -> ValueBox<T>;
}
impl IntoBox for ValuePointer {
    /// Take an untyped raw pointer and convert it into a box with a given expected type.
    unsafe fn into_box<T: Any + Sized>(self) -> ValueBox<T> {
        Box::from_raw(self as *mut T)
    }
}

pub trait IntoPointer {
    unsafe fn into_pointer(self) -> ValuePointer;
}
impl<T: Any> IntoPointer for ValueBox<T> {
    /// Get the untyped raw pointer for a given typed, boxed value.
    unsafe fn into_pointer(self) -> ValuePointer {
        mem::transmute(self)
    }
}

/// Primitive functions must be wrapped in `Box` since the size of `Fn` is not known at
/// compile time.
pub type BoxedPrimitiveFn = Box<Fn(&mut Machine, &Frame)>;

/// Wrapper around `BoxedPrimitiveFn` so that we can implement traits on it
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

/// Maps keys (fully-qualified paths) to various values (consts, statics, defined functions, and
/// primitive functions)
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
        let value = self.table.get(symbol);

        match value {
            Some(v) => v,
            None => panic!("Symbol not found: {:?}", symbol),
        }
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

    /// Instruction pointer (address of the instruction to be/being executed)
    pub ip: Addr,

    pub stack: Vec<ValuePointer>,

    pub symbol_table: SymbolTable,
}

/// Frame on the call stack
pub struct Frame {
    pub return_addr: Addr,
    pub args: Vec<ValuePointer>,
    pub slots: Vec<ValuePointer>,
}

/// Ways for modules to be loaded into machines.
pub trait ModuleLoad {
    /// Load a compiled module into a machine. Performs the following operations:
    ///
    /// - Adds module's bytecode to the machine's program data storage
    /// - Adds module's exported symbols (functions, consts, statics) to machine's symbol table
    /// - Resolves the modules relocations into concrete addresses/indices
    fn load_module(&mut self, compiled: &CompiledModule);
}

impl ModuleLoad for Machine {
    fn load_module(&mut self, compiled: &CompiledModule) {
        use super::super::asm_compiler::CompiledRelocationTarget::*;

        let ref relocations = compiled.relocations;

        let base_addr = self.code.len() as u64;
        self.code.extend(compiled.code.clone());

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
