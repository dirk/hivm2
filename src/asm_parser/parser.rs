#![allow(dead_code)]

use nom::{alpha, digit, eof, is_space, space, IResult, Needed};
use std::str;
use asm::{
    Assignment,
    AssignmentOp,
    Const,
    Extern,
    Local,
    Mod,
    Path,
    Program,
    Return,
    Static,
    Statement,
};

/// `u8` byte array that all parsing functions use for input/remaining parse subject data.
pub type PBytes<'a> = &'a[u8];

/// Alias for Nom's `IResult` that includes the `PBytes` byte array type alias for subject data.
pub type PResult<'a, O> = IResult<PBytes<'a>, O>;

/// Convert a byte array to a heap-allocated `String`.
fn to_s(i: PBytes) -> String {
    // String::from_utf8_lossy(i).into_owned()
    str::from_utf8(i).unwrap().to_string()
}

pub fn pprogram(input: &[u8]) -> IResult<&[u8], Program> {
    let result = chain!(input,
        stmts: many0!(pstatement) ~
        pterminal?                ,

        ||{ Program::with_stmts(stmts) }
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
        pmod    => { |m| Statement::StatementMod(m)    } |
        pextern => { |e| Statement::StatementExtern(e) } |
        pconst  => { |c| Statement::StatementConst(c)  } |
        pstatic => { |s| Statement::StatementStatic(s) } |
        plocal  => { |l| Statement::StatementLocal(l)  }
    )
}

named!(plocal_name<PBytes, String>,
    map!(alpha, |name| { to_s(name) })
);

named!(pstatic_name<PBytes, String>,
    map!(preceded!(tag!("$"), alpha), |name| { "$".to_string() + &to_s(name) })
);

named!(pconst_name<PBytes, String>,
    map!(preceded!(tag!("@"), alpha), |name| { "@".to_string() + &to_s(name) })
);

/// Parses a path like "a.b.c"
pub fn ppath(input: PBytes) -> PResult<Path> {
    map!(input,
        separated_nonempty_list!(tag!("."), alpha), |raw_segments: Vec<&[u8]>| {
            let segments = raw_segments.iter().map(|s| to_s(s) ).collect();

            Path::new(segments)
        }
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

named!(_const_constructor_pair<&[u8], (Path, Option<String>)>,
    chain!(
        cons: ppath ~ space ~
        arg:  alt!(
                  pterminal                               => { |_| None } |
                  terminated!(pconst_argument, pterminal) => { |arg| Some(arg) }
              ),

        ||{ (cons, arg) }
    )
);

/// Parses `const @NAME = CONSTRUCTOR ARGUMENT?``
pub fn pconst(input: &[u8]) -> IResult<&[u8], Const> {
    chain!(input,
        tag!("const")               ~ space ~
        name: pconst_name           ~ space ~
        tag!("=")                   ~ space? ~
        cp: _const_constructor_pair ~
        pterminal                   ,

        ||{
            let cons = cp.0.clone();
            let arg  = cp.1.clone();

            Const::new(name, cons, arg)
        }
    )
}

/// Parses constant constructor (string, number or null)
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

named!(_identifier<&[u8], String>,
    alt!(
        pconst_name |
        pstatic_name |
        plocal_name
    )
);

/// Parses assignments in the following forms:
///
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
        rvalue: _identifier                      ~
        pterminal                                ,

        ||{
            let op = AssignmentOp::from_str(str::from_utf8(raw_op).unwrap()).unwrap();

            Assignment::new(lvalue, op, rvalue)
        }
    )
}

fn gobble<F: Fn(u8) -> bool>(input: &[u8], test: F) -> &[u8] {
    for (index, item) in input.iter().enumerate() {
        if !test(*item) {
            return &input[index..]
        }
    }

    input
}

pub fn pterminal(input: PBytes) -> PResult<()> {
    let input = gobble(input, is_space);

    // Allow EOF to count as a terminal but DON'T consume it if it matches
    match eof(input) {
        IResult::Done(_, _) => { return IResult::Done(input, ()) },
        _ => (),
    }

    map!(input, tag!("\n"), { |_| () })
}

/// Parses the two patterns for returns:
///
/// - `return`
/// - `return ARGUMENT`
pub fn preturn(input: &[u8]) -> IResult<&[u8], Return> {
    chain!(input,
        tag!("return") ~
        arg: alt!(
                 pterminal                                 => { |_| None } |
                 delimited!(space, _identifier, pterminal) => { |arg| Some(arg) }
             ),

        ||{ Return::new(arg) }
    )
}

#[cfg(test)]
mod tests {
    use super::{passignment, pconst, plocal, ppath, pprogram, preturn, pstatic};
    use nom::{IResult};
    use asm::*;

    const EMPTY: &'static [u8] = b"";

    // Create a `IResult::Done` with no remaining input and the given output.
    fn done<T>(output: T) -> IResult<&'static [u8], T> {
        IResult::Done(EMPTY, output)
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
    fn parse_assignment() {
        let parsed_assignment = passignment(b"a = b");

        let expected_assignment = Assignment::new(
            "a".to_string(),
            AssignmentOp::Plain,
            "b".to_string()
        );

        assert_eq!(parsed_assignment, IResult::Done(EMPTY, expected_assignment))
    }

    #[test]
    fn parse_allocate_and_assign() {
        let parsed_assignment = passignment(b"a := b");

        let expected_assignment = Assignment::new(
            "a".to_string(),
            AssignmentOp::AllocateAndAssign,
            "b".to_string()
        );

        assert_eq!(parsed_assignment, IResult::Done(EMPTY, expected_assignment))
    }

    #[test]
    fn parse_return_with_argument() {
        let parsed_return   = preturn(b"return foo");
        let expected_return = Return::new(Some("foo".to_string()));

        assert_eq!(parsed_return, IResult::Done(EMPTY, expected_return))
    }

    #[test]
    fn parse_return_without_argument() {
        let parsed_return   = preturn(b"return");
        let expected_return = Return::new(None);

        assert_eq!(parsed_return, IResult::Done(EMPTY, expected_return))
    }

    #[test]
    fn parse_trivial_programs() {
        // Totally empty program
        assert_eq!(
            pprogram(b""),
            IResult::Done(EMPTY, Program::new())
        );

        let l = Local::new("foo".to_string());
        let p = Program::with_stmts(vec![Statement::StatementLocal(l)]);

        // Without a trailing newline before EOF
        assert_eq!(
            pprogram(b"local foo"),
            IResult::Done(EMPTY, p.clone())
        );

        // With a trailing newline before EOF
        assert_eq!(
            pprogram(b"local foo\n"),
            IResult::Done(EMPTY, p.clone())
        )
    }

    #[test]
    fn parse_basic_program() {
        let mut expected_program = Program::new();

        expected_program.push_mod(Mod::new(Path::with_name("foo".to_string())));
        expected_program.push_static(Static::new("$bar".to_string()));

        assert_eq!(
            pprogram(b"mod foo\nstatic $bar"),
            IResult::Done(EMPTY, expected_program)
        )
    }

    #[test]
    fn tolerates_whitespace_before_statements() {
        let m = Mod::new(Path::with_name("foo".to_string()));
        let p = Program::with_stmts(vec![Statement::StatementMod(m)]);

        assert_eq!(pprogram(b" \tmod foo"), done(p))
    }
}
