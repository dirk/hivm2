#![allow(dead_code)]

#[derive(PartialEq)]
pub struct Program {
    stmts: Vec<Statement>
}

impl Program {
    fn new() -> Program {
        Program {
            stmts: Vec::new(),
        }
    }

    fn push_mod(&mut self, m: Mod) {
        self.stmts.push(Statement::StatementMod(m));
    }

    fn push_extern(&mut self, e: Extern) {
        self.stmts.push(Statement::StatementExtern(e));
    }

    fn push_static(&mut self, s: Static) {
        self.stmts.push(Statement::StatementStatic(s));
    }
}

#[derive(Clone, PartialEq)]
struct BasicBlock {
    stmts: Vec<Statement>
}

#[derive(Clone, PartialEq)]
pub enum Statement {
    StatementMod(Mod),
    StatementExtern(Extern),
    StatementConst(Const),
    StatementStatic(Static),
    StatementLocal(Local),
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

#[derive(Debug)]
enum ParseError { }

pub type Name = String;

/// Represents a period-separated list of names.
#[derive(Clone, PartialEq)]
pub struct Path {
    segments: Vec<Name>,
}

impl Path {
    fn with_name(name: Name) -> Path {
        Path { segments: vec![name], }
    }

    fn from_str(s: &str) -> Result<Path, ParseError> {
        let parts = s.split('.');
        let segments = parts.map(|p| p.to_string() ).collect();

        Ok(Path { segments: segments })
    }
}

#[derive(Clone, PartialEq)]
pub struct Mod {
    path: Path,
}

impl Mod {
    fn new(path: Path) -> Mod {
        Mod { path: path }
    }
}

#[derive(Clone, PartialEq)]
pub struct Extern {
    path: Path,
}

impl Extern {
    fn new(path: Path) -> Extern {
        Extern { path: path }
    }
}

#[derive(Clone, PartialEq)]
struct Const {
    name: Name,
    constructor: Path,
    argument: Option<String>,
}

impl Const {
    fn new(name: Name, constructor: Path, argument: Option<String>) -> Const {
        Const {
            name: name,
            constructor: constructor,
            argument: argument,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Static {
    name: Name,
}

#[derive(Clone, PartialEq)]
struct Local {
    name: Name,
}

#[derive(Clone, PartialEq)]
struct Fn {
    name: Name,
    parameters: Vec<Name>,
    body: BasicBlock,
}

impl Fn {
    fn new(name: Name, parameters: Vec<Name>, body: BasicBlock) -> Fn {
        Fn {
            name: name,
            parameters: parameters,
            body: body,
        }
    }
}

#[derive(Clone, PartialEq)]
struct Return {
    name: Option<Name>,
}

#[derive(Clone, PartialEq)]
struct Call {
    name: Name,
    arguments: Vec<Name>,
}

impl Call {
    fn new(name: Name, arguments: Vec<Name>) -> Call {
        Call {
            name: name,
            arguments: arguments,
        }
    }
}

#[derive(Clone, PartialEq)]
struct Test {
    name: Name,
}

#[derive(Clone, PartialEq)]
struct If {
    condition: BasicBlock,
    then_sibling: Then,
}

#[derive(Clone, PartialEq)]
struct Then {
    body: BasicBlock,
    else_sibling: Option<Else>
}

#[derive(Clone, PartialEq)]
struct Else {
    body: BasicBlock,
}

#[derive(Clone, PartialEq)]
struct While {
    body: BasicBlock,
    // Some if this While is the lead and it's followed by a Do
    do_sibling: Option<Box<Do>>,
}

#[derive(Clone, PartialEq)]
struct Do {
    body: BasicBlock,
    // Some if this Do is lead and it's followed by a While
    while_sibling: Option<Box<While>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_pushes<F>(block: F) where
        F: Fn(&mut Program) {

        let mut p = Program::new();

        block(&mut p);
        assert_eq!(p.stmts.len(), 1)
    }

    #[test]
    fn parse_path() {
        let p1 = Path::from_str("a").unwrap();
        assert_eq!(p1.segments, ["a"]);

        let p2 = Path::from_str("a.b").unwrap();
        assert_eq!(p2.segments, ["a", "b"])
    }

    #[test]
    fn create_program() {
        let p = Program::new();
        assert_eq!(p.stmts.len(), 0)
    }

    #[test]
    fn push_mod() {
        assert_pushes(|p: &mut Program| {
            let m = Mod::new(Path::from_str("test").unwrap());
            p.push_mod(m);
        })
    }

    #[test]
    fn push_extern() {
        assert_pushes(|p: &mut Program| {
            let e = Extern::new(Path::from_str("an_extern").unwrap());
            p.push_extern(e);
        })
    }

    #[test]
    fn push_static() {
        assert_pushes(|p: &mut Program| {
            let s = Static { name: "a_static".to_string() };
            p.push_static(s);
        })
    }
}
