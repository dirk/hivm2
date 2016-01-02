use asm;
use asm::Statement::*;
use asm::AssignmentOp;
use vm::bytecode::ops::*;

use std::io::Write;

type ByteVec = Vec<u8>;

trait Compile {
    fn compile(&self) -> ByteVec;
}

struct Locals<'a> {
    locals: Vec<&'a str>,
}

impl<'a> Locals<'a> {
    fn new() -> Locals<'a> {
        Locals { locals: vec![], }
    }

    fn add(&mut self, local: &'a str) -> Result<u16, String> {
        if self.locals.contains(&local) {
            return Err(format!("Local already exists: {:?}", local))
        }
        self.locals.push(local);
        Ok((self.locals.len() - 1) as u16)
    }

    fn len(&self) -> usize { self.locals.len() }
}


impl asm::Fn {
    fn collect_locals(&self) -> Locals {
        let mut locals = Locals::new();
        let ref stmts = self.body.stmts;

        for stmt in stmts {
            match stmt {
                &StatementAssignment(ref assg) => {
                    if assg.operator == AssignmentOp::AllocateAndAssign {
                        locals.add(&assg.lvalue).unwrap();
                    }
                },
                &StatementLocal(ref local) => {
                    locals.add(&local.name).unwrap();
                }
                _ => (),
            }
        }

        locals
    }
}

impl Compile for asm::Fn {
    fn compile(&self) -> ByteVec {
        let locals = self.collect_locals();
        let entry = BFnEntry { num_locals: locals.len() as u16, };

        let mut ops: Vec<BOp> = vec![];
        ops.push(entry.into_op());

        BOp::compile_ops(ops)
    }
}
