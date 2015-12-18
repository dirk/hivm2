#![allow(dead_code)]

struct Program {
    stmts: Vec<Statement>
}

struct BasicBlock {
    stmts: Vec<Statement>
}

enum Statement {
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

type Name = String;

struct Path {
    segments: Vec<Name>,
}

struct Mod {
    path: Path,
}

struct Extern {
    path: Path,
}

struct Const {
    name: Name,
    constructor: Path,
    argument: Option<String>,
}

struct Static {
    name: Name,
}

struct Local {
    name: Name,
}

struct Fn {
    name: Name,
    parameters: Vec<Name>,
    body: BasicBlock,
}

struct Return {
    name: Option<Name>,
}

struct Call {
    name: Name,
    arguments: Vec<Name>,
}

struct Test {
    name: Name,
}

struct If {
    condition: BasicBlock,
    then_sibling: Then,
}

struct Then {
    body: BasicBlock,
    else_sibling: Option<Else>
}

struct Else {
    body: BasicBlock,
}

struct While {
    body: BasicBlock,
    // Some if this While is the lead and it's followed by a Do
    do_sibling: Option<Box<Do>>,
}

struct Do {
    body: BasicBlock,
    // Some if this Do is lead and it's followed by a While
    while_sibling: Option<Box<While>>,
}
