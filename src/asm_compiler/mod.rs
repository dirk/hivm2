use asm;
use asm::Statement::*;
use asm::AssignmentOp;
use vm::bytecode::ops::*;

type ByteVec = Vec<u8>;
type OpVec = Vec<BOp>;

#[derive(Clone, Copy)]
struct LocalContext<'a> {
    locals: &'a Locals<'a>,
}

trait Compile {
    fn compile(&self, Option<LocalContext>) -> OpVec;
}

trait CompileToValue {
    /// Generate a series of ops guaranteeing the introduction of 1 value at the top of the
    /// stack (to be consumed by subsequent op).
    fn compile_to_value(&self, Option<LocalContext>) -> OpVec;
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

    fn find(&self, local: &str) -> Result<u16, String> {
        let result = self.locals.binary_search(&local);

        match result {
            Ok(idx) => Ok(idx as u16),
            Err(_) => Err(format!("Local not found: {:?}", local)),
        }
    }
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

    fn compile_body(&self, ops: &mut Vec<BOp>, locals: &Locals) {
        let ref stmts = self.body.stmts;

        let lc = LocalContext { locals: locals, };

        for stmt in stmts {
            let stmt_ops = stmt.compile(Some(lc));

            ops.extend(stmt_ops)
        };
    }
}

impl Compile for asm::Statement {
    fn compile(&self, lc: Option<LocalContext>) -> OpVec {
        match *self {
            // StatementMod(m)                 => m.compile(),
            // StatementExtern(e)              => e.compile(),
            // StatementConst(c)               => c.compile(),
            // StatementStatic(s)              => s.compile(),
            // StatementLocal(Local),
            StatementAssignment(ref a)          => a.compile(lc),
            // StatementDefn(Defn),
            StatementFn(ref f)                  => f.compile(lc),
            // StatementReturn(Return),
            // StatementCall(Call),
            // StatementTest(Test),
            // StatementIf(If),
            // StatementThen(Then),
            // StatementElse(Else),
            // StatementWhile(While),
            // StatementDo(Do),
            // StatementBreak
            _                              => OpVec::new(),
        }
    }
}

impl CompileToValue for asm::Value {
    fn compile_to_value(&self, _: Option<LocalContext>) -> OpVec {
        OpVec::new()
    }
}

impl Compile for asm::Assignment {
    fn compile(&self, lc: Option<LocalContext>) -> OpVec {
        let ref locals = lc.unwrap().locals;
        let mut ops: Vec<BOp> = vec![];

        let idx = locals.find(&self.lvalue).unwrap();

        ops.extend(self.rvalue.compile_to_value(lc));
        ops.push(BSetLocal { idx: idx, }.into_op());

        ops
    }
}

impl Compile for asm::Fn {
    fn compile(&self, _: Option<LocalContext>) -> OpVec {
        let locals = self.collect_locals();
        let entry = BFnEntry { num_locals: locals.len() as u16, };

        let mut ops: Vec<BOp> = vec![];
        ops.push(entry.into_op());

        self.compile_body(&mut ops, &locals);

        ops
    }
}
