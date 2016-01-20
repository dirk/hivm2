#![allow(dead_code)]

pub enum ValidationError {
    InvalidTopLevelStatement(Statement),
    MissingModStatement,
    MoreThanOneModStatement,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    pub stmts: Vec<Statement>
}

impl Module {
    pub fn new() -> Module {
        Module {
            stmts: Vec::new(),
        }
    }

    pub fn with_stmts(stmts: Vec<Statement>) -> Module {
        Module { stmts: stmts }
    }

    pub fn push_mod(&mut self, m: Mod) {
        self.stmts.push(Statement::StatementMod(m));
    }

    pub fn push_extern(&mut self, e: Extern) {
        self.stmts.push(Statement::StatementExtern(e));
    }

    pub fn push_static(&mut self, s: Static) {
        self.stmts.push(Statement::StatementStatic(s));
    }

    pub fn push_defn(&mut self, d: Defn) {
        self.stmts.push(Statement::StatementDefn(d));
    }

    /// Check that the module is properly formed and that it doesn't contain invalid statements.
    pub fn validate(&self) -> Result<(), ValidationError> {
        let ref stmts = self.stmts;
        let mut mod_statements = 0;

        for stmt in stmts {
            match stmt {
                &Statement::StatementExtern(_) |
                &Statement::StatementConst(_)  |
                &Statement::StatementStatic(_) |
                &Statement::StatementDefn(_) => (),
                &Statement::StatementMod(_) => {
                    mod_statements += 1
                }
                _ => {
                    return Err(ValidationError::InvalidTopLevelStatement(stmt.clone()))
                },
            }
        }

        if mod_statements == 0 {
            return Err(ValidationError::MissingModStatement)
        } else if mod_statements > 1 {
            return Err(ValidationError::MoreThanOneModStatement)
        }

        Ok(())
    }

}

#[derive(Clone, Debug, PartialEq)]
pub struct BasicBlock {
    pub stmts: Vec<Statement>
}

impl BasicBlock {
    fn new() -> BasicBlock {
        BasicBlock { stmts: vec![], }
    }

    pub fn with_stmts(stmts: Vec<Statement>) -> BasicBlock {
        BasicBlock { stmts: stmts, }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    StatementMod(Mod),
    StatementExtern(Extern),
    StatementConst(Const),
    StatementStatic(Static),
    StatementLocal(Local),
    StatementAssignment(Assignment),
    StatementDefn(Defn),
    StatementFn(Fn),
    StatementReturn(Return),
    StatementCall(Call),
    StatementTest(Test),
    StatementIf(If),
    StatementThen(Then),
    StatementElse(Else),
    StatementWhile(While),
    StatementDo(Do),
    StatementBreak,
}

/// Represents any node that can potentially act as a value in the assembly AST.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Name(Name),
    Path(Path),
    Fn(Fn),
    Call(Call),
}

impl Value {
    pub fn with_name(name: Name) -> Value {
        Value::Name(name)
    }

    pub fn from_name_str(s: &str) -> Value {
        Value::Name(s.to_string())
    }
}

#[derive(Debug)]
pub enum ParseError<'a> {
    InvalidOperator(&'a str),
    InvalidSegment(String),
}

pub type Name = String;

/// Represents a period-separated list of names.
#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    segments: Vec<Name>,
}

impl Path {
    pub fn new<'a>(segments: Vec<Name>) -> Result<Path, ParseError<'a>> {
        let len = segments.len();

        if len > 1 {
            // Get all elements before the last
            let front = segments[0..len - 1].to_vec();

            // Return error if any of the front segments are const or static
            for segment in front.iter() {
                if segment.starts_with("@") {
                    return Err(ParseError::InvalidSegment(
                        format!("Found constant '{:?}' inside Path", segment)
                    ))
                }

                if segment.starts_with("$") {
                    return Err(ParseError::InvalidSegment(
                        format!("Found static '{:?}' inside Path", segment)
                    ))
                }
            }
        }

        Ok(Path { segments: segments })
    }

    pub fn ends_with_const(&self) -> bool {
        self.segments.last().unwrap().starts_with("@")
    }

    pub fn ends_with_static(&self) -> bool {
        self.segments.last().unwrap().starts_with("$")
    }

    pub fn with_name(name: Name) -> Path {
        Path::new(vec![name]).unwrap()
    }

    pub fn from_str(s: &str) -> Result<Path, ParseError> {
        let parts = s.split('.');
        let segments = parts.map(|p| p.to_string() ).collect();

        return Path::new(segments)
    }
}

impl ToString for Path {
    fn to_string(&self) -> String {
        self.segments.join(".")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mod {
    pub path: Path,
}

impl Mod {
    pub fn new(path: Path) -> Mod {
        Mod { path: path }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Extern {
    path: Path,
}

impl Extern {
    pub fn new(path: Path) -> Extern {
        Extern { path: path }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Const {
    pub name: Name,
    pub constructor: Path,
    pub argument: Option<String>,
}

impl Const {
    pub fn new(name: Name, constructor: Path, argument: Option<String>) -> Const {
        Const {
            name: name,
            constructor: constructor,
            argument: argument,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Static {
    pub name: Name,
}

impl Static {
    pub fn new(name: Name) -> Static {
        Static { name: name }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Local {
    pub name: Name,
}

impl Local {
    pub fn new(name: Name) -> Local {
        Local { name: name }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AssignmentOp {
    Plain,
    AllocateAndAssign,
}

impl AssignmentOp {
    pub fn from_str(op: &str) -> Result<AssignmentOp, ParseError> {
        match op {
            "="  => Ok(AssignmentOp::Plain),
            ":=" => Ok(AssignmentOp::AllocateAndAssign),
            _    => Err(ParseError::InvalidOperator(op)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Assignment {
    pub lvalue: Name,
    pub operator: AssignmentOp,
    pub rvalue: Value,
}

impl Assignment {
    pub fn new(lvalue: Name, op: AssignmentOp, rvalue: Value) -> Assignment {
        Assignment {
            lvalue: lvalue,
            operator: op,
            rvalue: rvalue,
        }
    }
}

/// Represents a named function.
#[derive(Clone, Debug, PartialEq)]
pub struct Defn {
    pub name: Name,
    pub parameters: Vec<Name>,
    pub body: BasicBlock,
}

impl Defn {
    pub fn new(name: Name, parameters: Vec<Name>, body: BasicBlock) -> Defn {
        Defn {
            name: name,
            parameters: parameters,
            body: body,
        }
    }
}

/// Represents an anonymous function value.
#[derive(Clone, Debug, PartialEq)]
pub struct Fn {
    pub parameters: Vec<Name>,
    pub body: BasicBlock,
}

impl Fn {
    pub fn new(parameters: Vec<Name>, body: BasicBlock) -> Fn {
        Fn { parameters: parameters, body: body, }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Return {
    value: Option<Value>,
}

impl Return {
    pub fn new(value: Option<Value>) -> Return {
        Return { value: value }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    pub path: Path,
    pub arguments: Vec<Name>,
}

impl Call {
    pub fn new(path: Path, arguments: Vec<Name>) -> Call {
        Call {
            path: path,
            arguments: arguments,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Test {
    pub name: Name,
}

#[derive(Clone, Debug, PartialEq)]
pub struct If {
    pub condition: BasicBlock,
    pub then_sibling: Then,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Then {
    pub body: BasicBlock,
    pub else_sibling: Option<Else>
}

#[derive(Clone, Debug, PartialEq)]
pub struct Else {
    body: BasicBlock,
}

#[derive(Clone, Debug, PartialEq)]
pub struct While {
    body: BasicBlock,
    // Some if this While is the lead and it's followed by a Do
    do_sibling: Option<Box<Do>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Do {
    body: BasicBlock,
    // Some if this Do is lead and it's followed by a While
    while_sibling: Option<Box<While>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Fn as StdFn;

    fn assert_pushes<F>(block: F) where
        F: StdFn(&mut Module) {

        let mut p = Module::new();

        block(&mut p);
        assert_eq!(p.stmts.len(), 1)
    }

    #[test]
    fn parse_path() {
        let p1 = Path::from_str("a").unwrap();
        assert_eq!(p1.segments, ["a"]);

        let p2 = Path::from_str("a.b").unwrap();
        assert_eq!(p2.segments, ["a", "b"]);

        // Check that it parses one with a constant at the end.
        let p3 = Path::from_str("a.b.@c");
        let expected_p3 = Path { segments: vec!["a".to_string(), "b".to_string(), "@c".to_string()], };
        assert!(p3.is_ok());
        assert_eq!(p3.unwrap(), expected_p3)
    }

    #[test]
    fn errors_on_bad_path() {
        assert_eq!(Path::from_str("$a.b").is_err(), true);

        assert_eq!(Path::from_str("a.@b.c").is_err(), true)
    }

    #[test]
    fn create_module() {
        let p = Module::new();
        assert_eq!(p.stmts.len(), 0)
    }

    #[test]
    fn push_mod() {
        assert_pushes(|p: &mut Module| {
            let m = Mod::new(Path::from_str("test").unwrap());
            p.push_mod(m);
        })
    }

    #[test]
    fn push_extern() {
        assert_pushes(|p: &mut Module| {
            let e = Extern::new(Path::from_str("an_extern").unwrap());
            p.push_extern(e);
        })
    }

    #[test]
    fn push_static() {
        assert_pushes(|p: &mut Module| {
            let s = Static::new("a_static".to_string());
            p.push_static(s);
        })
    }

    #[test]
    fn push_defn() {
        assert_pushes(|p: &mut Module| {
            let bb = BasicBlock::new();
            let d = Defn::new("a_defn".to_string(), vec![], bb);
            p.push_defn(d);
        })
    }

}
