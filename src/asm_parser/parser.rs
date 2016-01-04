use super::util::*;
use asm::{
    Assignment,
    AssignmentOp,
    BasicBlock,
    Const,
    Defn,
    Extern,
    Fn as AsmFn,
    Local,
    Mod,
    Module,
    Path,
    Return,
    Static,
    Statement,
    Value,
};

use nom::{
    alpha, digit, eof, is_space, multispace, space,
    IResult, Needed
};
use std::str;

pub fn pmodule(input: &[u8]) -> IResult<&[u8], Module> {
    let result = chain!(input,
        stmts: many0!(pstatement) ~
        pterminal?                ,

        ||{ Module::with_stmts(stmts) }
    );

    match result {
        IResult::Done(remaining, _) => {
            if remaining.len() > 0 {
                IResult::Incomplete(Needed::Size(remaining.len()))
            } else {
                result
            }
        },
        _ => result
    }
}

pub fn pstatement(input: PBytes) -> PResult<Statement> {
    let input = gobble(input, is_space);

    alt!(input,
        pmod        => { |m| Statement::StatementMod(m)    } |
        pextern     => { |e| Statement::StatementExtern(e) } |
        pconst      => { |c| Statement::StatementConst(c)  } |
        pstatic     => { |s| Statement::StatementStatic(s) } |
        plocal      => { |l| Statement::StatementLocal(l)  } |
        preturn     => { |r| Statement::StatementReturn(r) } |

        // NOTE: Assignment must come last since it will consume any alphanumeric word.
        passignment => { |a| Statement::StatementAssignment(a) }
    )
}

fn plocal_name(input: PBytes) -> PResult<String> {
    map!(input, alpha, |name| { to_s(name) })
}

fn pstatic_name(input: PBytes) -> PResult<String> {
    map!(input,
        preceded!(tag!("$"), alpha),
        |name| { "$".to_string() + &to_s(name) }
    )
}

fn pconst_name(input: PBytes) -> PResult<String> {
    map!(input,
        preceded!(tag!("@"), alpha),
        |name| { "@".to_string() + &to_s(name) }
    )
}

/// Parses a path like "a.b.c"
pub fn ppath(input: PBytes) -> PResult<Path> {
    named!(name<PBytes, String>,
        alt!(plocal_name | pstatic_name | pconst_name)
    );

    map!(input,
        separated_nonempty_list!(tag!("."), name),
        |segments| { Path::new(segments).unwrap() }
    )
}

/// Parses a mod definition
pub fn pmod(input: &[u8]) -> IResult<&[u8], Mod> {
    chain!(input,
        tag!("mod") ~ space ~
        path: ppath ~
        pterminal   ,

        ||{ Mod::new(path) }
    )
}

/// Parses `local NAME`
pub fn plocal(input: &[u8]) -> IResult<&[u8], Local> {
    chain!(input,
        tag!("local")     ~
        space             ~
        name: plocal_name ~
        pterminal         ,

        ||{ Local::new(name) }
    )
}

/// Parses `static $NAME`
pub fn pstatic(input: &[u8]) -> IResult<&[u8], Static> {
    chain!(input,
        tag!("static")     ~
        space              ~
        name: pstatic_name ~
        pterminal          ,

        ||{ Static::new(name) }
    )
}

/// Parses `extern PATH` where path is like "foo.bar".
pub fn pextern(input: &[u8]) -> IResult<&[u8], Extern> {
    chain!(input,
        tag!("extern") ~
        space          ~
        path: ppath    ~
        pterminal      ,

        ||{ Extern::new(path) }
    )
}

/// Parses constant constructor (path to a function and an optional argument)
pub fn pconst_constructor(input: PBytes) -> PResult<(Path, Option<String>)> {
    fn maybe_arg(input: PBytes) -> PResult<Option<String>> {
        try(input, Box::new(|i| pconst_argument(i)))
    }

    chain!(input,
        cons: ppath     ~ space ~
        arg:  maybe_arg ~
        pterminal       ,

        ||{ (cons, arg) }
    )
}

/// Parses `const @NAME = CONSTRUCTOR ARGUMENT?``
pub fn pconst(input: &[u8]) -> IResult<&[u8], Const> {
    chain!(input,
        tag!("const")               ~ space ~
        name: pconst_name           ~ space ~
        tag!("=")                   ~ space? ~
        cons: pconst_constructor    ~
        pterminal                   ,

        ||{
            let path = cons.0.clone();
            let arg  = cons.1.clone();

            Const::new(name, path, arg)
        }
    )
}

/// Parses constant constructor argument (string, number or null)
///
/// - string = `"[^"]*"``
/// - number = `[0-9]+`
/// - null = `null`
pub fn pconst_argument(input: PBytes) -> PResult<String> {
    alt!(input,
        pconst_string |
        pconst_number |
        pconst_null
    )
}

named!(pconst_string<&[u8], String>,
    chain!(
        tag!("\"")               ~
        value: take_until!("\"") ~
        tag!("\"")               ,

        ||{ to_s(value) }
    )
);
named!(pconst_number<&[u8], String>,
    map!(digit, |value| { to_s(value) })
);
named!(pconst_null<&[u8], String>,
    map!(tag!("null"), |_| { "null".to_string() })
);

fn ppidentifier(input: PBytes) -> PResult<String> {
    alt!(input,
        pconst_name |
        pstatic_name |
        plocal_name
    )
}

/// Parses a value type:
/// - An anonymous function (`fn(ARGS) BLOCK`)
/// - An identifier (`local`, `@static`, or `$const`)
pub fn pvalue(input: PBytes) -> PResult<Value> {
    try_each(input, vec![
        Box::new(|i| map!(i, pfn, |f| Value::Fn(f))),
        Box::new(|i| map!(i, ppidentifier, |i| Value::with_name(i)))
    ])
}

/// Parses assignments in the following forms:
/// - `NAME = VALUE`
/// - `NAME := VALUE`
///
/// Where name can be a static or local storage and value can be any kind of storage identifier.
///
/// **Note:** Right now value can only be another name.
pub fn passignment(input: &[u8]) -> IResult<&[u8], Assignment> {
    chain!(input,
        lvalue: alt!(plocal_name | pstatic_name) ~ space ~
        raw_op: alt!(tag!(":=") | tag!("="))     ~ space ~
        rvalue: pvalue                          ~
        pterminal                                ,

        ||{
            let op = AssignmentOp::from_str(str::from_utf8(raw_op).unwrap()).unwrap();

            Assignment::new(lvalue, op, rvalue)
        }
    )
}

pub fn pterminal(input: PBytes) -> PResult<()> {
    let input = gobble(input, is_space);

    named!(right_brace, tag!("}"));

    if peek(input, eof) {
        return IResult::Done(input, ())
    }
    if peek(input, right_brace) {
        return IResult::Done(input, ())
    }

    map!(input, tag!("\n"), { |_| () })
}

/// Parses a block: `{ STATEMENTS }`.
fn pbasicblock(input: PBytes) -> PResult<BasicBlock> {
    chain!(input,
        tag!("{")                 ~ multispace? ~
        stmts: many0!(pstatement) ~ multispace? ~
        tag!("}") ,

        ||{ BasicBlock::with_stmts(stmts) }
    )
}

fn ppfunction_parameters(input: PBytes) -> PResult<Vec<String>> {
    // Comma separator between parameters
    named!(comma<&[u8], ()>,
        chain!(
            opt!(space) ~ tag!(",") ~ opt!(space),
            ||{ () }
        )
    );

    chain!(input,
        tag!("(")                                  ~ space? ~
        args: separated_list!(comma, ppidentifier) ~ space? ~
        tag!(")")                                  ,

        ||{ args }
    )
}

/// Parses the `defn` statement syntax for defined functions.
pub fn pdefn(input: PBytes) -> PResult<Defn> {
    chain!(input,
        tag!("defn")                      ~ space ~
        name: alpha                       ~
        parameters: ppfunction_parameters ~ space? ~
        body: pbasicblock                 ,

        ||{ Defn::new(to_s(name), parameters, body) }
    )
}

/// Parses the `fn` value syntax for anonymous functions.
pub fn pfn(input: PBytes) -> PResult<AsmFn> {
    chain!(input,
        tag!("fn")                        ~ space? ~
        parameters: ppfunction_parameters ~ space? ~
        body: pbasicblock                 ,

        ||{ AsmFn::new(parameters, body) }
    )
}

/// Parses the two patterns for returns:
/// - `return`
/// - `return VALUE`
pub fn preturn(input: PBytes) -> PResult<Return> {
    fn maybe_arg(input: PBytes) -> PResult<Option<Value>> {
        try(input, Box::new(|i| preceded!(i, space, pvalue)))
    }

    chain!(input,
        tag!("return") ~
        arg: maybe_arg ~
        pterminal      ,

        ||{ Return::new(arg) }
    )
}

#[cfg(test)]
mod tests {
    use super::{
        passignment, pbasicblock, pconst, pdefn, plocal, ppath, pmodule, preturn, pstatic
    };
    use super::super::util::{PBytes};
    use nom::{IResult};
    use asm::*;

    const EMPTY: &'static [u8] = b"";

    // Create a `IResult::Done` with no remaining input and the given output.
    fn done<'a, T>(output: T) -> IResult<PBytes<'a>, T> {
        IResult::Done(EMPTY, output)
    }

    fn unwrap_iresult<O>(result: IResult<&[u8], O>) -> O {
        match result {
            IResult::Done(_, output)    => output,
            IResult::Error(error)       => panic!("called `unwrap_iresult()` on an `Error` value: {:?}", error),
            IResult::Incomplete(needed) => panic!("called `unwrap_iresult()` on an `Incomplete` value: {:?}", needed),
        }
    }

    #[test]
    fn parse_path() {
        assert_eq!(ppath(b"a"), IResult::Done(EMPTY, Path::from_str("a").unwrap()));

        assert_eq!(ppath(b"b.c"), IResult::Done(EMPTY, Path::from_str("b.c").unwrap()))
    }

    #[test]
    fn parse_local() {
        let l = plocal(b"local foo");

        assert_eq!(l, IResult::Done(EMPTY, Local::new("foo".to_string())))
    }

    #[test]
    fn parse_static() {
        let s = pstatic(b"static $bar");

        assert_eq!(s, IResult::Done(EMPTY, Static::new("$bar".to_string())))
    }

    #[test]
    fn parse_const_with_argument() {
        let parsed_const = pconst(b"const @a = b \"c\"");

        let expected_const = Const::new(
            "@a".to_string(),
            Path::with_name("b".to_string()),
            Some("c".to_string())
        );

        assert_eq!(parsed_const, IResult::Done(EMPTY, expected_const))
    }

    #[test]
    fn parse_const_without_argument() {
        let parsed_const = pconst(b"const @a = b");

        let expected_const = Const::new(
            "@a".to_string(),
            Path::with_name("b".to_string()),
            None
        );

        assert_eq!(parsed_const, IResult::Done(EMPTY, expected_const))
    }

    #[test]
    fn parse_assignment_with_name() {
        let parsed_assignment = passignment(b"a = b");

        let expected_assignment = Assignment::new(
            "a".to_string(),
            AssignmentOp::Plain,
            Value::from_name_str("b")
        );

        assert_eq!(parsed_assignment, IResult::Done(EMPTY, expected_assignment))
    }

    #[test]
    fn parse_assignment_with_fn() {
        let parsed_assignment = passignment(b"a = fn() { b := @c }");

        let expected_assignment = Assignment::new(
            "a".to_owned(),
            AssignmentOp::Plain,
            Value::Fn(Fn::new(
                Vec::new(),
                BasicBlock::with_stmts(vec![
                    Statement::StatementAssignment(unwrap_iresult(passignment(b"b := @c")))
                ])
            ))
        );

        assert_eq!(parsed_assignment, done(expected_assignment))
    }

    #[test]
    fn parse_allocate_and_assign() {
        let parsed_assignment = passignment(b"a := b");

        let expected_assignment = Assignment::new(
            "a".to_string(),
            AssignmentOp::AllocateAndAssign,
            Value::from_name_str("b")
        );

        assert_eq!(parsed_assignment, IResult::Done(EMPTY, expected_assignment))
    }

    #[test]
    fn parse_basic_block_with_right_brace_as_terminal() {
        let expected_bb = BasicBlock::with_stmts(vec![
            Statement::StatementLocal(Local::new("bar".to_string()))
        ]);

        assert_eq!(
            pbasicblock(b"{local bar}"),
            done(expected_bb)
        )
    }

    #[test]
    fn parse_defn_without_params() {
        let body = BasicBlock::with_stmts(vec![
            Statement::StatementAssignment(unwrap_iresult(passignment(b"bar := baz")))
        ]);

        let expected_defn = Defn::new(
            "foo".to_string(),
            vec![],
            body
        );

        let parsed_defn = pdefn(b"defn foo() {\n bar := baz \n}");

        assert_eq!(
            parsed_defn,
            done(expected_defn)
        )
    }

    #[test]
    fn parse_return_with_argument() {
        let parsed_return   = preturn(b"return foo");
        let expected_return = Return::new(Some(Value::from_name_str("foo")));

        assert_eq!(parsed_return, IResult::Done(EMPTY, expected_return))
    }

    #[test]
    fn parse_return_without_argument() {
        let parsed_return   = preturn(b"return");
        let expected_return = Return::new(None);

        assert_eq!(parsed_return, IResult::Done(EMPTY, expected_return))
    }

    #[test]
    fn parse_trivial_modules() {
        // Totally empty module
        assert_eq!(
            pmodule(b""),
            IResult::Done(EMPTY, Module::new())
        );

        let l = Local::new("foo".to_string());
        let m = Module::with_stmts(vec![Statement::StatementLocal(l)]);

        // Without a trailing newline before EOF
        assert_eq!(
            pmodule(b"local foo"),
            done(m.clone())
        );

        // With a trailing newline before EOF
        assert_eq!(
            pmodule(b"local foo\n"),
            done(m)
        )
    }

    #[test]
    fn parse_basic_module() {
        let mut expected_module = Module::new();

        let m = Mod::new(Path::with_name("foo".to_string()));
        let s = Static::new("$bar".to_string());
        expected_module.push_mod(m);
        expected_module.push_static(s);

        assert_eq!(
            pmodule(b"mod foo\nstatic $bar"),
            done(expected_module)
        )
    }

    #[test]
    fn tolerates_whitespace_before_statements() {
        let m = Mod::new(Path::with_name("foo".to_string()));
        let m = Module::with_stmts(vec![Statement::StatementMod(m)]);

        assert_eq!(pmodule(b" \tmod foo"), done(m))
    }
}
