pub type ValuePointer = u64;

/// The actual virtual machine
pub struct Machine {
    /// Bytecode stored in the virtual machine
    pub code: Vec<u8>,

    pub call_stack: Vec<Frame>,

    pub ip: ValuePointer,

    pub stack: Vec<u64>,
}

pub struct Frame {
    pub return_addr: ValuePointer,
    pub args: Vec<ValuePointer>,
    pub slots: Vec<u64>,
}
