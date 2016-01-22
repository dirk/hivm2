use asm;
use asm::Statement::*;
use asm::AssignmentOp;
use vm::bytecode::ops::*;

use std::fmt::Debug;
use std::hash::{Hash, Hasher};
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

#[derive(Clone, Debug)]
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

    ConstPath(String),
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

#[derive(Clone, Debug)]
pub enum FunctionName {
    Named(String),
    Anonymous
}

/// Representation of named and anonymous functions in a compiled module.
#[derive(Clone, Debug)]
pub struct Function {
    pub name: FunctionName,
    pub ops: OpVec,
}

/// 3-tuple of the name, constructor path, and optional argument.
pub type CompiledConst = (String, String, Option<String>);

pub struct Module {
    /// Fully-qualified name of the module
    pub name: String,
    /// All the relocations in this module
    pub relocations: Vec<Relocation>,
    /// All the functions in this module
    pub functions: Vec<Rc<Function>>,
    pub consts: Vec<CompiledConst>,
    pub statics: Vec<String>,
}

trait PointerPartialEq {
    fn pointer_eq(&self, other: &Self) -> bool {
        (self as *const Self) == (other as *const Self)
    }
}
impl PointerPartialEq for BOp {}
impl PointerPartialEq for Function {}

trait PointerHash : Debug {
    fn pointer_hash<H: Hasher>(&self, state: &mut H) {
        let ptr = (self as *const Self) as *const usize;
        // println!("pointer_hash: {:?} -> {:?}", self, ptr as u64);
        state.write_u64(ptr as u64)
    }
}
impl PointerHash for BOp {}
impl PointerHash for Function {}

/// Custom equality for BOp; we're comparing pointers instead of values because values can be
/// identical for different BOps in the op stream.
impl PartialEq for BOp {
    fn eq(&self, other: &BOp) -> bool {
        self.pointer_eq(other)
    }
}
impl Eq for BOp {}

/// Custom hash for BOp; using their raw pointers since their the value of different BOps in
/// the op stream can be the same.
impl Hash for BOp {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pointer_hash(state)
    }
}

/// Function values may also be the same (they have the same op stream), so we're comparing
/// them based on their pointers.
impl PartialEq for Function {
    fn eq(&self, other: &Function) -> bool {
        self.pointer_eq(other)
    }
}
impl Eq for Function {}

/// Functions should be distinct in the hash based on their location in memory.
impl Hash for Function {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pointer_hash(state)
    }
}

impl Module {
    pub fn new() -> Module {
        Module {
            name: "".to_owned(),
            relocations: vec![],
            functions: vec![],
            consts: vec![],
            statics: vec![],
        }
    }

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

    fn add_const_relocation(&mut self, site: Rc<BOp>, target: String) {
        self.relocations.push(Relocation {
            site: site,
            target: RelocationTarget::ConstPath(target),
        })
    }
}

pub enum CompiledRelocationTarget {
    InternalAddress(u64),
    ExternalFunctionPath(String),
    ConstPath(String),
}

pub struct CompiledModule {
    pub name: String,
    pub code: Vec<u8>,
    pub functions: Vec<(String, u64)>,
    pub consts: Vec<CompiledConst>,
    pub statics: Vec<String>,
    pub relocations: Vec<(u64, CompiledRelocationTarget)>,
}

use std::collections::HashMap;
use std::borrow::Borrow;

pub type OpMap = HashMap<Rc<BOp>, u64>;
pub type FunctionMap = HashMap<Rc<Function>, u64>;
pub type CompiledRelocationVec = Vec<(u64, CompiledRelocationTarget)>;

pub trait CompileModule {
    fn compile(&self) -> CompiledModule;
}

impl CompileModule for asm::Module {
    fn compile(&self) -> CompiledModule {
        let mut module = Module::new();

        let mut op_map: OpMap                 = HashMap::new();
        let mut function_map: FunctionMap     = HashMap::new();
        let mut code: Vec<u8>                 = Vec::new();
        let mut functions: Vec<(String, u64)> = Vec::new();

        // Compile and ingest the top-level module statements
        {
            let mut module_ops = OpVec::new();
            let ref stmts = self.stmts;
            for stmt in stmts {
                module_ops.extend(stmt.compile(None, &mut module))
            }
            self.ingest_ops(&mut code, module_ops, &mut op_map);
        }

        // Ingest all the compiled functions; track their entry addresses in `function_map` and
        // in the module's symbol list
        for f in module.functions {
            // This will be the address of the `BFnEntry` op
            let addr = code.len() as u64;
            function_map.insert(f.clone(), addr);

            if let FunctionName::Named(ref name) = f.name {
                functions.push((name.clone(), addr))
            }

            let function_ops = f.ops.clone();
            self.ingest_ops(&mut code, function_ops, &mut op_map);
        }

        let relocations = self.resolve_relocations(module.relocations, &op_map, &function_map);

        CompiledModule {
            name: module.name,
            code: code,
            functions: functions,
            consts: module.consts,
            statics: module.statics,
            relocations: relocations,
        }
    }
} // impl CompileModule for asm::Module

impl asm::Module {
    /// Take a vector of higher-level owned and shared `Op`s and compile them down to bytecode.
    /// Also notes the module-local addresses of shared `Op`s for later relocation in an `OpMap`.
    pub fn ingest_ops(&self, bytecode: &mut Vec<u8>, ops: OpVec, op_map: &mut OpMap) {
        for op in ops {
            match op {
                Op::Owned(op) => bytecode.extend(op.to_binary()),
                Op::Shared(shared) => {
                    // Length of the vec will be the first address of the op we're inserting
                    let addr = bytecode.len() as u64;
                    op_map.insert(shared.clone(), addr);

                    let op: &BOp = shared.borrow();
                    bytecode.extend(op.clone().to_binary())
                },
            }
        }
    }

    /// Resolves abstract relocations (`Relocation`) into a vector of concrete, address-based
    /// relocations (`CompiledRelocationVec`) suitable for loading and linking into a
    /// virtual machine instance.
    pub fn resolve_relocations(&self, relocations: Vec<Relocation>, op_map: &OpMap, function_map: &FunctionMap) -> CompiledRelocationVec {
        // Resolve all the relocations
        let mut compiled_relocations: CompiledRelocationVec = Vec::new();

        for relocation in relocations {
            let site = relocation.site;

            let site_base_address = match op_map.get(&site) {
                Some(addr) => addr,
                None => panic!("Site not found: {:?}", site),
            };

            // Right now all ops have a max of 1 address
            let site_address = site_base_address + site.addr_field_offset(0);

            let compiled = match relocation.target {
                RelocationTarget::InternalBranchAddress(op) => {
                    let target_addr = op_map.get(&op).unwrap();
                    (
                        site_address,
                        CompiledRelocationTarget::InternalAddress(*target_addr)
                    )
                },
                RelocationTarget::InternalFunctionAddress(fref) => {
                    let target_addr = function_map.get(&fref).unwrap();
                    (
                        site_address,
                        CompiledRelocationTarget::InternalAddress(*target_addr)
                    )
                },
                RelocationTarget::ExternalFunctionPath(path) => {
                    (
                        site_address,
                        CompiledRelocationTarget::ExternalFunctionPath(path)
                    )
                },
                RelocationTarget::ConstPath(path) => {
                    (
                        site_address,
                        CompiledRelocationTarget::ConstPath(path)
                    )
                }
            };

            compiled_relocations.push(compiled)
        }

        compiled_relocations
    }

}// impl asm::Module

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
            StatementConst(ref c)       => c.compile(lc, m),
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
            _                           => panic!("Compile#compile not implemented for {:?}", self),
        }
    }
}

impl Compile for asm::Mod {
    fn compile(&self, _: LocalContextRef, m: &mut Module) -> OpVec {
        let fully_qualified_name = self.path.to_string();

        if m.name.is_empty() {
            m.name = fully_qualified_name
        } else {
            panic!("Cannot redefine module: {:?}", m.name)
        }

        vec![]
    }
}

impl Compile for asm::Const {
    fn compile(&self, _: LocalContextRef, m: &mut Module) -> OpVec {
        let compiled = (self.name.clone(), self.constructor.to_string(), self.argument.clone());
        m.consts.push(compiled);
        vec![]
    }
}

impl Compile for asm::Static {
    fn compile(&self, _: LocalContextRef, m: &mut Module) -> OpVec {
        m.statics.push(self.name.clone());
        vec![]
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
            asm::Value::Path(ref p) => p.compile_to_value(lc, m),
            // _                    => panic!("#compile_to_value not implemented for {:?}", self),
        }
    }
}

impl CompileToValue for asm::Path {
    fn compile_to_value(&self, _: LocalContextRef, m: &mut Module) -> OpVec {
        let op: BOp =
            if self.ends_with_const() {
                BLoadConst { id: 0, }.into_op()
            } else {
                panic!("Cannot compile Path to value: {:?}", self)
            };

        let shared_op = Rc::new(op);
        m.add_const_relocation(shared_op.clone(), self.to_string());

        let mut ops = OpVec::new();
        ops.push_shared(shared_op);
        ops
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
        m.add_call_relocation(op.clone(), self.path.to_string());

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
        m.add_defn(Function {
            name: FunctionName::Named(self.name.clone()),
            ops: ops,
        });

        vec![]
    }
}

impl CompileToValue for asm::Fn {
    fn compile_to_value(&self, _: LocalContextRef, m: &mut Module) -> OpVec {
        let ops  = compile_function_body(&self.body, m);
        let fref = m.add_fn(Function {
            name: FunctionName::Anonymous,
            ops: ops,
        });

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

#[cfg(test)]
mod tests {
    use super::{CompileModule};
    use asm::{BasicBlock, Defn, Module, Return, Statement};

    #[test]
    fn test_compile_module() {
        let module = Module::with_stmts(vec![
            Statement::StatementDefn(Defn::new(
                "a".to_owned(),
                vec![],
                BasicBlock::with_stmts(vec![
                    Statement::StatementReturn(Return::new(None))
                ])
            )),
        ]);
        let compiled = module.compile();

        assert!(compiled.code.len() > 0);
        assert_eq!(compiled.functions.len(), 1);
    }
}
