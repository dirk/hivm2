use nom::{ErrorKind, Err as NomErr, IResult};
use std::str;

/// `u8` byte array that all parsing functions use for input/remaining parse subject data.
pub type PBytes<'a> = &'a[u8];

/// Alias for Nom's `IResult` that includes the `PBytes` byte array type alias for subject data.
pub type PResult<'a, O> = IResult<PBytes<'a>, O>;

/// A boxed standard parser function that takes a byte array as input and returns a standard
/// parsing result.
pub type TryFn<'a, T> = Box<Fn(PBytes<'a>) -> PResult<'a, T>>;

/// Tries each of a given set of matchers, returning the first one that matches successfully.
/// If all fail then it returns an `IResult::Error` at the position where it failed.
pub fn try_each<'a, T>(input: PBytes<'a>, matchers: Vec<TryFn<'a, T>>) -> PResult<'a, T> {
    for matcher in matchers.iter() {
        let result = matcher(input);

        match result {
            IResult::Done(_, _) => { return result },
            _ => (),
        }
    }

    return IResult::Error(
        NomErr::Position(ErrorKind::Alt, input)
    )
}

/// Tries the given matcher function; if it succeeds then it consumes the input and returns the
/// corresponding `IResult::Done` with `Some(output)`, for all other cases (error, incomplete) it
/// does not consume any input but still returns `IResult::Done` with a `None` output.
pub fn try<'a, T>(input: PBytes<'a>, matcher: TryFn<'a, T>) -> PResult<'a, Option<T>> {
    let result = matcher(input);

    match result {
        IResult::Done(rest, output) => { IResult::Done(rest, Some(output)) },
        _ => { IResult::Done(input, None) },
    }
}

/// Convert a byte array to a heap-allocated `String`.
pub fn to_s(i: PBytes) -> String {
    // String::from_utf8_lossy(i).into_owned()
    str::from_utf8(i).unwrap().to_string()
}

pub fn gobble<F: Fn(u8) -> bool>(input: PBytes, test: F) -> PBytes {
    for (index, item) in input.iter().enumerate() {
        if !test(*item) {
            return &input[index..]
        }
    }

    input
}

// Peek to see if the next input matches the given function WITHOUT consuming the input.
pub fn peek<F>(input: PBytes, f: F) -> bool
    where F: Fn(PBytes) -> IResult<PBytes, PBytes> {

    match f(input) {
        IResult::Done(_, _) => true,
        _ => false
    }
}

#[cfg(test)]
mod tests {
    use super::{
        gobble,
        peek,
        try,
        PBytes,
        PResult
    };
    use nom::{
        is_space,
        IResult
    };

    #[test]
    fn peek_does_peek_ahead() {
        fn matcher(input: PBytes) -> PResult<PBytes> {
            tag!(input, "ab")
        }

        assert_eq!(peek(b"abc", matcher), true);
        assert_eq!(peek(b"cde", matcher), false);
    }

    #[test]
    fn gobble_consumes_input() {
        assert_eq!(gobble(b" \tab", is_space), b"ab");
    }

    #[test]
    fn try_consumes_if_matches() {
        assert_eq!(
            try(b"abc", Box::new(|i| tag!(i, "ab"))),
            IResult::Done("c".as_bytes(), Some("ab".as_bytes()))
        )
    }

    #[test]
    fn try_doesnt_consume_if_doesnt_match() {
        assert_eq!(
            try(b"abc", Box::new(|i| tag!(i, "cd"))),
            IResult::Done("abc".as_bytes(), None)
        )
    }
}
