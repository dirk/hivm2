use super::super::asm;
use super::super::asm_compiler::{CompiledRelocationTarget, CompileModule};
use super::bytecode::util::NativeEndianWriteExt;

use std::collections::HashMap;
use std::io::Cursor;

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

impl TableValue {
    fn as_u64(&self) -> u64 {
        match *self {
            TableValue::Const(vp) => vp,
            TableValue::Static(vp) => vp,
            TableValue::Defn(ptr) => ptr,
        }
    }
}

pub struct SymbolTable {
    table: HashMap<TableKey, TableValue>,
}

impl SymbolTable {
    fn has_symbol(&self, symbol: &TableKey) -> bool {
        self.table.contains_key(symbol)
    }

    fn lookup_symbol(&self, symbol: &TableKey) -> &TableValue {
        self.table.get(symbol).unwrap()
    }
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

trait ModuleLoad {
    fn load_module(&mut self, &mut asm::Module);
}

impl ModuleLoad for Machine {
    fn load_module(&mut self, module: &mut asm::Module) {
        use super::super::asm_compiler::CompiledRelocationTarget::*;

        let compiled = module.compile();
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
                        let target_addr = target.as_u64();
                        writer.write_hu64(target_addr);
                    } else {
                        panic!("Symbol not found in symbol table: {:?}", path)
                    }
                },
            }
        }
    }// fn load_module
}
