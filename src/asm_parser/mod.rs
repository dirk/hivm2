#![allow(dead_code)]

use nom::{alpha, space};
use asm::{Extern, Local, Path, Static};
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
        tag!("$")   ~
        name: alpha ,

        ||{ "$".to_string() + &to_s(name) }
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
                segments.push(to_s(name));
            }

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

#[cfg(test)]
mod tests {
    use super::plocal;
    use nom::{IResult};
    use asm::*;

    #[test]
    fn parse_local() {
        let l = plocal(b"local foo");

        assert_eq!(l, IResult::Done(&b""[..], Local::new("foo".to_string())))
    }
}
