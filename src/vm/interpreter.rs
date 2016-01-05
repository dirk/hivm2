use super::machine::{Frame, Machine};

use std::io::{Cursor, Seek, SeekFrom};

pub trait Execute {
    fn execute(&mut self);
}

impl Machine {
    fn get_stack_top_mut(&mut self) -> &mut Frame {
        self.call_stack.last_mut().unwrap()
    }

    fn get_stack_top(&self) -> &Frame {
        self.call_stack.last().unwrap()
    }
}

impl Execute for Machine {
    fn execute(&mut self) {
        use super::bytecode::ops::*;
        use super::bytecode::ops::BOp::*;

        let code = self.code.clone();

        let mut cursor = Cursor::new(&code);
        cursor.seek(SeekFrom::Start(self.ip)).unwrap();

        loop {
            let op = BOp::from_binary(&mut cursor);
            let mut next_addr = cursor.position();

            match op {
                FnEntry(fn_entry) => {
                    let mut frame = self.get_stack_top_mut();
                    frame.slots.resize(fn_entry.num_locals as usize, 0x0);
                },
                GetLocal(get_local) => {
                    let value: u64;
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
                    // TODO: Pop args off the stack and put them in the `Frame`
                    let frame = Frame {
                        slots: Vec::new(),
                        return_addr: next_addr,
                    };
                    self.call_stack.push(frame);
                    next_addr = call.addr;
                },
                Invoke(_) => {
                    // TODO: Correctly implement invoke (ie. pop args and put in frame)
                    let frame = Frame {
                        slots: Vec::new(),
                        return_addr: next_addr,
                    };
                    self.call_stack.push(frame);
                    next_addr = self.stack.pop().unwrap();
                },
                PushAddress(push_address) => {
                    self.stack.push(push_address.addr);
                },
                BranchIf(branch_if) => {
                    let value = self.stack.pop().unwrap();
                    if value == 0x0 {
                        next_addr = branch_if.dest
                    }
                },
                BranchIfNot(branch_if_not) => {
                    let value = self.stack.pop().unwrap();
                    if value != 0x0 {
                        next_addr = branch_if_not.dest
                    }
                },
                Return => {
                    // TODO: Implement return
                },
                Pop => {
                    self.stack.pop().unwrap();
                },
                Noop => {},
            };

            self.ip = next_addr;
        } // loop
    }

} // impl Machine
