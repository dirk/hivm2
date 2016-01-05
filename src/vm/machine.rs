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

trait ModuleLoad {
    fn load_module(&mut self, &mut asm::Module);
}

impl ModuleLoad for Machine {
    fn load_module(&mut self, module: &mut asm::Module) {
        use super::super::asm_compiler::CompiledRelocationTarget::*;

        let compiled = module.compile();

        let base_addr = self.code.len() as u64;
        self.code.extend(compiled.code);

        let mut writer = Cursor::new(&mut self.code[..]);

        let ref relocations = compiled.relocations;
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
                    panic!("Cannot handle named relocation: {:?}", path)
                },
            }
        }
    }
}
