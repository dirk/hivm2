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

pub type Name = String;

#[derive(Clone, PartialEq)]
pub struct Path {
    segments: Vec<Name>,
}

impl Path {
    fn with_name(name: Name) -> Path {
        Path { segments: vec![name], }
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
struct Extern {
    path: Path,
}

#[derive(Clone, PartialEq)]
struct Const {
    name: Name,
    constructor: Path,
    argument: Option<String>,
}

#[derive(Clone, PartialEq)]
struct Static {
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

#[derive(Clone, PartialEq)]
struct Return {
    name: Option<Name>,
}

#[derive(Clone, PartialEq)]
struct Call {
    name: Name,
    arguments: Vec<Name>,
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

    #[test]
    fn create_program() {
        let mut p = Program::new();
        assert_eq!(p.stmts.len(), 0);

        let m = Mod::new(Path::with_name("test".to_string()));
        p.push_mod(m);
        assert_eq!(p.stmts.len(), 1);
    }
}
