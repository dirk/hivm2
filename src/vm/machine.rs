use asm_compiler::{
    CompiledConst,
    CompiledModule,
    CompiledRelocationTarget,
};
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

use std::rc::Rc;

/// Primitive functions must be wrapped in `Box` since the size of `Fn` is not known at
/// compile time.
pub type BoxedPrimitiveFn = Rc<Fn(&mut Machine, &Frame)>;

/// Wrapper around `BoxedPrimitiveFn` so that we can implement traits on it
#[derive(Clone)]
pub struct PrimitiveFn(BoxedPrimitiveFn);

impl PrimitiveFn {
    fn call(&self, machine: &mut Machine, frame: &Frame) {
        let ref f = self.0;

        f(machine, frame)
    }
}

impl fmt::Debug for PrimitiveFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[native code]")
    }
}

pub type TableKey = String;

#[derive(Clone, Debug)]
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
#[derive(Clone)]
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

    pub fn lookup_symbol(&self, symbol: &TableKey) -> &TableValue {
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

type ConstConstructor<'a> = (String, &'a PrimitiveFn, Option<String>);

impl Machine {
    fn empty() -> Machine {
        Machine {
            code: vec![],
            call_stack: vec![],
            ip: 0,
            stack: vec![],
            symbol_table: SymbolTable::new(),
        }
    }

    fn load_consts(&mut self, compiled_module: &CompiledModule) {
        let ref consts = compiled_module.consts;
        let ref module_name = compiled_module.name;

        // Constructors are called on an empty machine instance because it's unsafe to let
        // them work with ourselves
        let mut empty = Machine::empty();

        // Immutable copy of the symbol table for resolving currently-existing symbols
        let static_symbol_table = self.symbol_table.clone();

        let calls = Machine::resolve_const_constructors(&static_symbol_table, consts.clone());

        for call in calls {
            let (const_name, constructor, argument) = call;

            // Build a fully-qualified name
            let mut name = String::new();
            name.push_str(&module_name);
            name.push_str(".");
            name.push_str(&const_name);

            let boxed_argument = ValueBox::new(argument);

            let frame = Frame {
                return_addr: 0,
                slots: vec![],
                args: vec![
                    unsafe { boxed_argument.into_pointer() },
                ],
            };

            constructor.call(&mut empty, &frame);

            let value = match empty.stack.pop() {
                Some(v) => v,
                None => panic!("Const constructor did not push a value for {:?}", name)
            };

            println!("Adding const: {:?}", name);

            self.symbol_table.set_symbol(&name, TableValue::Const(value));

        }
    }

    fn resolve_const_constructors(symbol_table: &SymbolTable, consts: Vec<CompiledConst>) -> Vec<ConstConstructor> {
        let mut constructors = vec![];

        for compiled_const in consts {
            let (name, constructor_path, argument) = compiled_const;

            let constructor = match symbol_table.lookup_symbol(&constructor_path) {
                &TableValue::Primitive(ref primitive_fn) => primitive_fn,
                _ => {
                    panic!("Const constructor not found: {:?}", constructor_path)
                },
            };

            constructors.push((name, constructor, argument));
        }

        return constructors
    }
}

impl ModuleLoad for Machine {
    fn load_module(&mut self, compiled: &CompiledModule) {
        use super::super::asm_compiler::CompiledRelocationTarget::*;

        self.load_consts(compiled);

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
                &ConstPath(ref path) => {
                    let is_local = path.starts_with("@") || path.starts_with("$");

                    let path: String =
                        if is_local {
                            compiled.name.clone()+"."+&path
                        } else {
                            path.clone()
                        };

                    let _ = self.symbol_table.lookup_symbol(&path);

                    // TODO: Write the address of the symbol
                }
            }
        }
    }// fn load_module
}
