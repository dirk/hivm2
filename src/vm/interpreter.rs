use super::machine::{
    Frame,
    IntoBox,
    IntoPointer,
    Machine,
    SymbolTable,
    TableValue,
    ValueBox,
    ValuePointer
};
use super::bytecode::types::Addr;

use std::any::Any;
use std::io::{Cursor};

pub trait Execute {
    fn execute(&mut self);
}

fn builtin_println(_: &mut Machine, f: &Frame) {
    let arg1 = *unsafe { f.args[0].into_box::<String>() };

    if !(&arg1 as &Any).is::<String>() {
        panic!("Expected argument 1 to be String, got {:?}", arg1)
    }

    println!("{}", arg1);
}

impl Machine {
    pub fn new() -> Machine {
        Machine {
            code: vec![],
            call_stack: vec![],
            ip: 0x0,
            stack: vec![],
            symbol_table: SymbolTable::new(),
        }
    }

    pub fn add_std(&mut self) {
        self.symbol_table.set_symbol(&"_.std.println".to_owned(), TableValue::with_fn(Box::new(builtin_println)));
    }

    #[inline]
    fn get_stack_top_mut(&mut self) -> &mut Frame {
        self.call_stack.last_mut().unwrap()
    }

    #[inline]
    fn get_stack_top(&self) -> &Frame {
        self.call_stack.last().unwrap()
    }

    /// Pop `num` entries off the top of the stack into a `Vec`. The first item in the vector
    /// will be the lowest item on the stack and the last item in the vector will be the highest
    /// (ie. at the top) of the stack.
    #[inline]
    fn pop_stack_into_vec(&mut self, num: usize) -> Vec<ValuePointer> {
        let mut out: Vec<ValuePointer> = Vec::with_capacity(num);

        for idx in (0..num).rev() {
            let value = self.stack.pop().unwrap();
            out[idx] = value
        }

        out
    }

    /// Pop `num_args` off the stack and build a stack frame with the given `return_addr`.
    #[inline]
    fn build_frame(&mut self, return_addr: u64, num_args: usize) -> Frame {
        Frame {
            return_addr: return_addr,
            args: self.pop_stack_into_vec(num_args),
            slots: Vec::new(),
        }
    }
}

impl Execute for Machine {
    fn execute(&mut self) {
        use super::bytecode::ops::*;
        use super::bytecode::ops::BOp::*;

        let code = self.code.clone();

        let mut cursor = Cursor::new(&code);
        cursor.set_position(self.ip);

        loop {
            let op = BOp::from_binary(&mut cursor);
            let mut next_addr = cursor.position();

            match op {
                FnEntry(fn_entry) => {
                    let mut frame = self.get_stack_top_mut();
                    frame.slots.resize(fn_entry.num_locals as usize, 0x0 as ValuePointer);
                },
                GetLocal(get_local) => {
                    let value: ValuePointer;
                    {
                        let frame = self.get_stack_top();
                        value = frame.slots[get_local.idx as usize];
                    }
                    self.stack.push(value);
                },
                SetLocal(set_local) => {
                    let value = self.stack.pop().unwrap();
                    let frame = self.get_stack_top_mut();
                    frame.slots[set_local.idx as usize] = value;
                },
                Call(call) => {
                    let frame = self.build_frame(next_addr, call.num_args as usize);
                    self.call_stack.push(frame);
                    next_addr = call.addr;
                },
                Invoke(invoke) => {
                    let frame = self.build_frame(next_addr, invoke.num_args as usize);
                    self.call_stack.push(frame);

                    // Get the boxed address value off the stack and jump to it
                    let value = self.stack.pop().unwrap();
                    let addr: ValueBox<Addr> = unsafe { value.into_box() };
                    next_addr = *addr;
                },
                PushAddress(push_address) => {
                    let boxed: ValueBox<Addr> = ValueBox::new(push_address.addr);
                    self.stack.push(unsafe { boxed.into_pointer() });
                },
                BranchIf(branch_if) => {
                    let value = self.stack.pop().unwrap();
                    if value.is_null() {
                        next_addr = branch_if.dest
                    }
                },
                BranchIfNot(branch_if_not) => {
                    let value = self.stack.pop().unwrap();
                    if !value.is_null() {
                        next_addr = branch_if_not.dest
                    }
                },
                Return => {
                    let frame = self.call_stack.pop().unwrap();
                    next_addr = frame.return_addr;
                },
                Pop => {
                    self.stack.pop().unwrap();
                },
                Noop => {},
            };

            self.ip = next_addr;
            cursor.set_position(next_addr);
        } // loop
    }

} // impl Execute for Machine
