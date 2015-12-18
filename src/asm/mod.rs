#![allow(dead_code)]

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

#[derive(PartialEq)]
struct BasicBlock {
    stmts: Vec<Statement>
}

#[derive(PartialEq)]
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

#[derive(PartialEq)]
pub struct Path {
    segments: Vec<Name>,
}

impl Path {
    fn with_name(name: Name) -> Path {
        Path { segments: vec![name], }
    }
}

#[derive(PartialEq)]
pub struct Mod {
    path: Path,
}

impl Mod {
    fn new(path: Path) -> Mod {
        Mod { path: path }
    }
}

#[derive(PartialEq)]
struct Extern {
    path: Path,
}

#[derive(PartialEq)]
struct Const {
    name: Name,
    constructor: Path,
    argument: Option<String>,
}

#[derive(PartialEq)]
struct Static {
    name: Name,
}

#[derive(PartialEq)]
struct Local {
    name: Name,
}

#[derive(PartialEq)]
struct Fn {
    name: Name,
    parameters: Vec<Name>,
    body: BasicBlock,
}

#[derive(PartialEq)]
struct Return {
    name: Option<Name>,
}

#[derive(PartialEq)]
struct Call {
    name: Name,
    arguments: Vec<Name>,
}

#[derive(PartialEq)]
struct Test {
    name: Name,
}

#[derive(PartialEq)]
struct If {
    condition: BasicBlock,
    then_sibling: Then,
}

#[derive(PartialEq)]
struct Then {
    body: BasicBlock,
    else_sibling: Option<Else>
}

#[derive(PartialEq)]
struct Else {
    body: BasicBlock,
}

#[derive(PartialEq)]
struct While {
    body: BasicBlock,
    // Some if this While is the lead and it's followed by a Do
    do_sibling: Option<Box<Do>>,
}

#[derive(PartialEq)]
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
