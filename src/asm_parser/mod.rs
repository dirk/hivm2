#![allow(dead_code)]

use nom::{alpha, space};
use asm::{Local, Static};

fn to_s(i: &[u8]) -> String {
    String::from_utf8_lossy(i).into_owned()
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

named!(plocal<&[u8], Local>,
    chain!(
        tag!("local")     ~
        space             ~
        name: plocal_name ,

        ||{ Local::new(name) }
    )
);

named!(pstatic<&[u8], Static>,
    chain!(
        tag!("static")     ~
        space              ~
        name: pstatic_name ,

        ||{ Static::new(name) }
    )
);
