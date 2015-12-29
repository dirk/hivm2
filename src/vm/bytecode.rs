#![allow(dead_code)]

/// Register index
type Reg = u8;

/// Address of a function
type Addr = u64;

/// Index of a local variable slot
type Local = u16;

struct BCall<'a> {
    /// Address of the function to be called
    addr: Addr,
    /// Argument registers of the call
    args: &'a [Reg],
    /// Output register for the return of the call
    out: Option<Reg>,
}

struct BReturn {
    arg: Option<Reg>,
}

struct BSetLocal {
    idx: Local,
    arg: Reg,
}

struct BGetLocal {
    idx: Local,
    out: Reg,
}

struct BEntry {
    /// Number of local variable slots
    num_local: u16,
}

struct BGetArg {
    /// Index of the argument, pass 255 to get the total number of arguments passed
    idx: u8,
    out: Reg,
}
