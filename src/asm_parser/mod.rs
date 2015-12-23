#![allow(dead_code)]

use nom::{alpha, digit, eof, space};
use asm::{Const, Extern, Local, Path, Return, Static};
use std::str;

fn to_s(i: &[u8]) -> String {
    // String::from_utf8_lossy(i).into_owned()
    str::from_utf8(i).unwrap().to_string()
}

named!(plocal_name<&[u8], String>,
    map!(alpha, |name| { to_s(name) })
);

named!(pstatic_name<&[u8], String>,
    chain!(
        tag!("$") ~ name: alpha,

        ||{ "$".to_string() + &to_s(name) }
    )
);

named!(pconst_name<&[u8], String>,
    chain!(
        tag!("@") ~ name: alpha,

        ||{ "@".to_string() + &to_s(name) }
    )
);

/// Parses "a.b.c"
named!(ppath<&[u8], Path>,
    chain!(
        head: alpha                                               ~
        rest: many0!(chain!(tag!(".") ~ name: alpha, ||{ name })) ,

        ||{
            let mut segments = vec![to_s(head)];

            for name in rest {
                segments.push(to_s(name))
            }

            println!("ppath: {:?}", segments);

            Path::new(segments)
        }
    )
);

/// Parses "local NAME"
named!(plocal<&[u8], Local>,
    chain!(
        tag!("local")     ~
        space             ~
        name: plocal_name ,

        ||{ Local::new(name) }
    )
);

/// Parses "static $NAME"
named!(pstatic<&[u8], Static>,
    chain!(
        tag!("static")     ~
        space              ~
        name: pstatic_name ,

        ||{ Static::new(name) }
    )
);

/// Parses "extern path1.path2"
named!(pextern<&[u8], Extern>,
    chain!(
        tag!("extern") ~
        space          ~
        path: ppath    ,

        ||{ Extern::new(path) }
    )
);

/// Parses constant constructor (string, number or null)
///
/// - string = "[^"]*"
/// - number = [0-9]+
/// - null = null
named!(pconst_argument<&[u8], String>,
    alt!(
        pconst_string |
        pconst_number |
        pconst_null
    )
);
named!(pconst_string<&[u8], String>,
    chain!(
        tag!("\"")               ~
        value: take_until!("\"") ~
        tag!("\"")               ,

        ||{ to_s(value) }
    )
);
named!(pconst_number<&[u8], String>,
    chain!(
        value: digit, ||{ to_s(value) }
    )
);
named!(pconst_null<&[u8], String>,
    chain!(
        tag!("null"), ||{ "null".to_string() }
    )
);

named!(pterminal<&[u8], ()>,
    chain!(
        space? ~
        eof    ,

        ||{ () }
    )
);

named!(pconst_constructor_pair<&[u8], (Path, Option<String>) >,
    chain!(
        cons: ppath ~ space ~
        arg:  alt!(
                  pterminal                               => { |_| None } |
                  terminated!(pconst_argument, pterminal) => { |arg| Some(arg) }
              ),

        ||{ (cons, arg) }
    )
);

/// Parses "const @NAME = constructor argument?"
named!(pconst<&[u8], Const>,
    chain!(
        tag!("const")               ~ space ~
        name: pconst_name           ~ space ~
        tag!("=")                   ~ space? ~
        cp: pconst_constructor_pair ,

        ||{
            let cons = cp.0.clone();
            let arg  = cp.1.clone();

            Const::new(name, cons, arg)
        }
    )
);

named!(preturn_arg<&[u8], String>,
    preceded!(space, map!(alpha, |name| { to_s(name) }))
);

/// Parses "return"
named!(preturn<&[u8], Return>,
    chain!(
        tag!("return")         ~
        arg: opt!(preturn_arg) ~
        pterminal              ,

        ||{ Return::new(arg) }
    )
);

#[cfg(test)]
mod tests {
    use super::{pconst, plocal};
    use nom::{IResult};
    use asm::*;

    #[test]
    fn parse_local() {
        let l = plocal(b"local foo");

        assert_eq!(l, IResult::Done(EMPTY, Local::new("foo".to_string())))
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

    const EMPTY: &'static [u8] = b"";

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
}
