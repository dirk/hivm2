use asm;
use asm::Statement::*;
use asm::AssignmentOp;
use vm::bytecode::ops::*;

use std::cell::RefCell;
use std::rc::Rc;

type ByteVec = Vec<u8>;
pub type OpVec = Vec<Op>;

trait OpVecExt {
    fn push_op(&mut self, BOp);
}
impl OpVecExt for OpVec {
    fn push_op(&mut self, op: BOp) {
        self.push(Op::BOp(op))
    }
}

#[derive(Clone)]
enum Op {
    BOp(BOp),
    BOpRef(RefCell<BOp>),
}

#[derive(Clone)]
struct Locals {
    locals: Vec<String>,
}

#[derive(Clone)]
struct LocalContext {
    locals: Locals,
}

type LocalContextRef<'a> = Option<&'a LocalContext>;

trait Compile {
    fn compile(&self, LocalContextRef, &mut Module) -> OpVec;
}

trait CompileToValue {
    /// Generate a series of ops guaranteeing the introduction of 1 value at the top of the
    /// stack (to be consumed by subsequent op).
    fn compile_to_value(&self, LocalContextRef, &mut Module) -> OpVec;
}

impl Locals {
    fn new() -> Locals {
        Locals { locals: vec![], }
    }

    fn add(&mut self, local: String) -> Result<u16, String> {
        if self.locals.contains(&local) {
            return Err(format!("Local already exists: {:?}", local))
        }
        self.locals.push(local);
        Ok((self.locals.len() - 1) as u16)
    }

    fn len(&self) -> usize { self.locals.len() }

    fn find(&self, local: String) -> Result<u16, String> {
        let result = self.locals.binary_search(&local);

        match result {
            Ok(idx) => Ok(idx as u16),
            Err(_) => Err(format!("Local not found: {:?}", local)),
        }
    }
}

pub enum RelocationTarget {
    Function(Rc<Function>),
}

pub struct Relocation {
    /// Site that must have its address relocated
    pub site: RefCell<BOp>,
    /// Point that this call site is referring to
    pub target: RelocationTarget,
}

pub struct Module {
    pub relocations: Vec<Relocation>,
    /// All the functions in this module
    pub functions: Vec<Rc<Function>>,
}

/// Representation of named and anonymous functions in a compiled module.
#[derive(Clone)]
pub struct Function {
    pub ops: OpVec,
}

impl Module {
    fn add_fn(&mut self, f: Function) -> Rc<Function> {
        let fref = Rc::new(f);
        self.functions.push(fref.clone());
        fref
    }
    fn add_defn(&mut self, f: Function) -> Rc<Function> {
        let fref = Rc::new(f);
        self.functions.push(fref.clone());
        fref
    }

    fn add_function_relocation(&mut self, site: RefCell<BOp>, target: Rc<Function>) {
        self.relocations.push(Relocation {
            site: site,
            target: RelocationTarget::Function(target),
        })
    }
}

impl asm::BasicBlock {
    fn collect_locals(&self) -> Locals {
        let mut locals = Locals::new();
        let ref stmts = self.stmts;

        for stmt in stmts {
            match stmt {
                &StatementAssignment(ref assg) => {
                    if assg.operator == AssignmentOp::AllocateAndAssign {
                        locals.add(assg.lvalue.clone()).unwrap();
                    }
                },
                &StatementLocal(ref local) => {
                    locals.add(local.name.clone()).unwrap();
                }
                _ => (),
            }
        }

        locals
    }
}

impl Compile for asm::BasicBlock {
    fn compile(&self, lc: LocalContextRef, m: &mut Module) -> OpVec {
        let ref stmts = self.stmts;
        let mut ops = OpVec::new();

        for stmt in stmts {
            ops.extend(stmt.compile(lc, m))
        }

        ops
    }
}

impl Compile for asm::Statement {
    fn compile(&self, lc: LocalContextRef, m: &mut Module) -> OpVec {
        match *self {
            StatementMod(_)             => vec![],
            // StatementExtern(e)       => e.compile(),
            // StatementConst(c)        => c.compile(),
            // StatementStatic(s)       => s.compile(),
            StatementLocal(_)           => vec![], // No-op since we'll have already collected locals
            StatementAssignment(ref a)  => a.compile(lc, m),
            StatementDefn(ref d)        => d.compile(lc, m),
            // StatementReturn(Return),
            // StatementCall(Call),
            // StatementTest(Test),
            // StatementIf(If),
            // StatementThen(Then),
            // StatementElse(Else),
            // StatementWhile(While),
            // StatementDo(Do),
            // StatementBreak
            _                           => panic!("#compile not implemented for {:?}", self),
        }
    }
}

fn get_local(name: asm::Name, lc: LocalContextRef, _: &mut Module) -> OpVec {
    let idx = lc.unwrap().locals.find(name).unwrap();

    vec![Op::BOp(BGetLocal { idx: idx, }.into_op())]
}

impl CompileToValue for asm::Value {
    fn compile_to_value(&self, lc: LocalContextRef, m: &mut Module) -> OpVec {
        match *self {
            asm::Value::Name(ref n) => get_local(n.clone(), lc, m),
            asm::Value::Fn(ref f)   => f.compile_to_value(lc, m),
            _                       => panic!("#compile_to_value not implemented for {:?}", self),
        }
    }
}

impl Compile for asm::Assignment {
    fn compile(&self, lc: LocalContextRef, m: &mut Module) -> OpVec {
        let idx = lc.unwrap().locals.find(self.lvalue.clone()).unwrap();
        let mut ops: OpVec = vec![];

        ops.extend(self.rvalue.compile_to_value(lc, m));
        ops.push_op(BSetLocal { idx: idx, }.into_op());

        ops
    }
}

fn compile_function_body(body: &asm::BasicBlock, m: &mut Module) -> OpVec {
    let locals = body.collect_locals();
    let entry  = BFnEntry { num_locals: locals.len() as u16, };

    let mut ops: OpVec = vec![];
    ops.push_op(entry.into_op());

    let lc = LocalContext { locals: locals, };
    ops.extend(body.compile(Some(&lc.clone()), m));

    ops
}

impl Compile for asm::Defn {
    fn compile(&self, _: LocalContextRef, m: &mut Module) -> OpVec {
        let ops = compile_function_body(&self.body, m);
        m.add_defn(Function { ops: ops, });

        vec![]
    }
}

impl CompileToValue for asm::Fn {
    fn compile_to_value(&self, _: LocalContextRef, m: &mut Module) -> OpVec {
        let ops  = compile_function_body(&self.body, m);
        let fref = m.add_fn(Function { ops: ops, });

        // Using `RefCell` so that we have a shared mutable pointer with which we can later
        // update the op with the correct address.
        let op = RefCell::new(BPushAddress { addr: 0, }.into_op());
        m.add_function_relocation(op.clone(), fref);

        vec![Op::BOpRef(op)]
    }
}
