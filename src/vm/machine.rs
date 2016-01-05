use std::collections::HashMap;

pub type ValuePointer = u64;

pub type TableKey = String;

pub enum TableValue {
    /// Pointer to the constant value in memory
    Const(ValuePointer),
    /// Pointer to the static value in memory
    Static(ValuePointer),
    /// Address in the machine's code for the function
    Defn(u64),
}

pub struct SymbolTable {
    table: HashMap<TableKey, TableValue>,
}

/// The actual virtual machine
pub struct Machine {
    /// Bytecode stored in the virtual machine
    pub code: Vec<u8>,

    pub call_stack: Vec<Frame>,

    pub ip: ValuePointer,

    pub stack: Vec<u64>,

    pub symbol_table: SymbolTable,
}

pub struct Frame {
    pub return_addr: ValuePointer,
    pub args: Vec<ValuePointer>,
    pub slots: Vec<u64>,
}
