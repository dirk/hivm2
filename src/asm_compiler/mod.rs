use asm;
use asm::Statement::*;
use asm::AssignmentOp;
use vm::bytecode::ops::*;

use std::rc::Rc;

type ByteVec = Vec<u8>;
pub type OpVec = Vec<Op>;

trait OpVecExt {
    fn push_owned(&mut self, BOp);
    fn push_shared(&mut self, Rc<BOp>);
}
impl OpVecExt for OpVec {
    fn push_owned(&mut self, op: BOp) {
        self.push(Op::Owned(op))
    }
    fn push_shared(&mut self, op: Rc<BOp>) {
        self.push(Op::Shared(op))
    }
}

#[derive(Clone)]
enum Op {
    Owned(BOp),
    Shared(Rc<BOp>),
}

/// Internal storage for all the locals in a LocalContext.
pub struct Locals {
    pub locals: Vec<String>,
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

/// Set of locals variables/slots and other values related to functions. Every function has its
/// own `LocalContext`.
pub struct LocalContext {
    pub locals: Locals,
}
pub type LocalContextRef<'a> = Option<&'a LocalContext>;

pub trait Compile {
    fn compile(&self, LocalContextRef, &mut Module) -> OpVec;
}

pub trait CompileToValue {
    /// Generate a series of ops guaranteeing the introduction of 1 value at the top of the
    /// stack (to be consumed by subsequent op).
    fn compile_to_value(&self, LocalContextRef, &mut Module) -> OpVec;
}

pub enum RelocationTarget {
    /// Internal address for a jump
    InternalBranchAddress(Rc<BOp>),
    /// Internal address of a function (for anonymous functions)
    InternalFunctionAddress(Rc<Function>),
    /// Absolute string version of the path to the external function
    ExternalFunctionPath(String),
}

/// Links sites in the code that need to have their addresses updated (relocated) with a
/// locator for where that address will eventually be.
///
/// At module load time the relocations of that module are scanned and the sites in the code
/// updated to point at the correct final address.
pub struct Relocation {
    /// Site that must have its address relocated
    pub site: Rc<BOp>,
    /// Where this site should eventually point to
    pub target: RelocationTarget,
}

pub struct Module {
    /// All the relocations in this module
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

    fn add_function_relocation(&mut self, site: Rc<BOp>, target: Rc<Function>) {
        self.relocations.push(Relocation {
            site: site,
            target: RelocationTarget::InternalFunctionAddress(target),
        })
    }

    fn add_call_relocation(&mut self, site: Rc<BOp>, target: String) {
        self.relocations.push(Relocation {
            site: site,
            target: RelocationTarget::ExternalFunctionPath(target),
        })
    }

    fn add_branch_relocation(&mut self, site: Rc<BOp>, target: Rc<BOp>) {
        self.relocations.push(Relocation {
            site: site,
            target: RelocationTarget::InternalBranchAddress(target),
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
            StatementCall(ref c)        => c.compile(lc, m),
            StatementReturn(ref r)      => r.compile(lc, m),
            StatementTest(ref t)        => t.compile(lc, m),
            StatementIf(ref i)          => i.compile(lc, m),
            StatementThen(_)            => vec![], // Both `then` and `else` are handled by `if`
            StatementElse(_)            => vec![],
            // StatementWhile(While),
            // StatementDo(Do),
            // StatementBreak
            _                           => panic!("#compile not implemented for {:?}", self),
        }
    }
}

impl asm::Value {
    fn compile_name_to_value(&self, name: asm::Name, lc: LocalContextRef, _: &mut Module) -> OpVec {
        let idx = lc.unwrap().locals.find(name).unwrap();

        vec![Op::Owned(BGetLocal { idx: idx, }.into_op())]
    }
}

impl CompileToValue for asm::Value {
    fn compile_to_value(&self, lc: LocalContextRef, m: &mut Module) -> OpVec {
        match *self {
            asm::Value::Name(ref n) => self.compile_name_to_value(n.clone(), lc, m),
            asm::Value::Fn(ref f)   => f.compile_to_value(lc, m),
            asm::Value::Call(ref c) => c.compile_to_value(lc, m),
            _                       => panic!("#compile_to_value not implemented for {:?}", self),
        }
    }
}

impl Compile for asm::Call {
    fn compile(&self, lc: LocalContextRef, m: &mut Module) -> OpVec {
        let mut ops = self.compile_to_value(lc, m);
        ops.push_owned(BOp::Pop); // Pop the value since it won't be used
        ops
    }
}
impl CompileToValue for asm::Call {
    fn compile_to_value(&self, lc: LocalContextRef, m: &mut Module) -> OpVec {
        let mut ops = OpVec::new();
        let ref args = self.arguments;

        for name in args {
            let idx = lc.unwrap().locals.find(name.clone()).unwrap();

            ops.push_owned(BGetLocal { idx: idx, }.into_op());
        }

        let num_args = self.arguments.len() as u8;
        let op = Rc::new(BCall { addr: 0, num_args: num_args, }.into_op());
        m.add_call_relocation(op.clone(), self.name.clone());

        ops.push_shared(op);
        ops
    }
}

impl Compile for asm::Return {
    fn compile(&self, _: LocalContextRef, _: &mut Module) -> OpVec {
        vec![Op::Owned(BOp::Return)]
    }
}

impl Compile for asm::Assignment {
    fn compile(&self, lc: LocalContextRef, m: &mut Module) -> OpVec {
        let idx = lc.unwrap().locals.find(self.lvalue.clone()).unwrap();
        let mut ops: OpVec = vec![];

        ops.extend(self.rvalue.compile_to_value(lc, m));
        ops.push_owned(BSetLocal { idx: idx, }.into_op());

        ops
    }
}

/// Shared function used by `asm::Fn` and `asm::Defn` to compile their `BasicBlock` bodies.
fn compile_function_body(body: &asm::BasicBlock, m: &mut Module) -> OpVec {
    let locals = body.collect_locals();
    let entry  = BFnEntry { num_locals: locals.len() as u16, };

    let mut ops: OpVec = vec![];
    ops.push_owned(entry.into_op());

    let lc = LocalContext { locals: locals, };
    ops.extend(body.compile(Some(&lc), m));

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

        // Using `Rc` so that we have a shared pointer that we can use to look up the op later
        let op = Rc::new(BPushAddress { addr: 0, }.into_op());
        m.add_function_relocation(op.clone(), fref);

        vec![Op::Shared(op)]
    }
}

impl asm::If {
    /// Compiles the body of the if condition. Since the `test` statement is last it will
    /// push a value to the top of the stack.
    fn compile_if_to_value(&self, lc: LocalContextRef, m: &mut Module) -> OpVec {
        let mut ops = self.condition.compile(lc, m);
        let entry = ops[0].clone();

        if let Op::Owned(plain_entry) = entry {
            ops.remove(0);
            ops.insert(0, Op::Shared(Rc::new(plain_entry.clone())));
        }

        ops
    }
}

impl Compile for asm::If {
    fn compile(&self, lc: LocalContextRef, m: &mut Module) -> OpVec {
        let mut ops = OpVec::new();

        let if_ops   = self.compile_if_to_value(lc, m);
        let then_ops = self.then_sibling.compile(lc, m);

        let branch_if_not = Rc::new(BBranchIf { dest: 0, }.into_op());
        let noop          = Rc::new(BOp::Noop);

        ops.extend(if_ops.clone());
        ops.push_shared(branch_if_not.clone()); // Branch to the noop if it fails
        ops.extend(then_ops.clone());
        ops.push_shared(noop.clone()); // Target if branch fails

        // Track that the branch-if-not needs to eventually point to the noop
        m.add_branch_relocation(branch_if_not, noop);

        ops
    }
}

impl Compile for asm::Then {
    fn compile(&self, lc: LocalContextRef, m: &mut Module) -> OpVec {
        self.body.compile(lc, m)
    }
}

/// **Note**: Test pushes its value onto the stack to be consumed by its condition
/// parent (if/while) node.
impl Compile for asm::Test {
    fn compile(&self, lc: LocalContextRef, _: &mut Module) -> OpVec {
        let name = self.name.clone();
        let idx = lc.unwrap().locals.find(name).unwrap();

        vec![Op::Owned(BGetLocal { idx: idx, }.into_op())]
    }
}
