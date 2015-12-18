#![allow(dead_code)]

struct BasicBlock {
    stmts: Vec<Statement>
}

enum Statement {
    StatementMod(Mod),
    StatementExtern(Extern),
    StatementConst(Const),
    StatementStatic(Static),
    StatementLocal(Local),
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
