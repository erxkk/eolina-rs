use nom::{
    bytes::complete::{tag, take_while},
    character::complete::digit1,
    combinator::opt,
    error::{Error as NomError, ErrorKind as NomErrorKind},
    multi::separated_list0,
    sequence::{delimited, pair},
    Err as NomErr, IResult,
};
use std::borrow::Cow;

// convinience re-export
pub use ignore_ascii_whitespace as aiw;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OneOf2<A, B> {
    A(A),
    B(B),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OneOf3<A, B, C> {
    A(A),
    B(B),
    C(C),
}

pub fn one_of_2<'a, F, O1, G, O2>(
    mut f: F,
    mut g: G,
) -> impl FnMut(&'a str) -> IResult<&'a str, OneOf2<O1, O2>>
where
    F: FnMut(&'a str) -> IResult<&'a str, O1>,
    G: FnMut(&'a str) -> IResult<&'a str, O2>,
{
    move |input| {
        f(input)
            .map(|(rest, output)| (rest, OneOf2::A(output)))
            .or_else(|_| g(input).map(|(rest, output)| (rest, OneOf2::B(output))))
    }
}

pub fn one_of_3<'a, F, G, H>(
    mut f: F,
    mut g: G,
    mut h: H,
) -> impl FnMut(&'a str) -> IResult<&'a str, OneOf3<&'a str, &'a str, &'a str>>
where
    F: FnMut(&'a str) -> IResult<&'a str, &'a str>,
    G: FnMut(&'a str) -> IResult<&'a str, &'a str>,
    H: FnMut(&'a str) -> IResult<&'a str, &'a str>,
{
    move |input| {
        f(input)
            .map(|(rest, output)| (rest, OneOf3::A(output)))
            .or_else(|_| {
                g(input)
                    .map(|(rest, output)| (rest, OneOf3::B(output)))
                    .or_else(|_| h(input).map(|(rest, output)| (rest, OneOf3::C(output))))
            })
    }
}

// explicit lifetimes needed
pub fn ignore_ascii_whitespace<'a, F>(f: F) -> impl Fn(&'a str) -> IResult<&'a str, &'a str>
where
    F: Fn(&'a str) -> IResult<&'a str, &'a str>,
{
    // delimited(
    //     take_while(|ch: char| ch.is_ascii_whitespace()),
    //     f,
    //     take_while(|ch: char| ch.is_ascii_whitespace()),
    // )
    move |input| {
        let (rest, _) = take_while(|ch: char| ch.is_ascii_whitespace())(input)?;
        let (rest, output) = f(rest)?;
        let (rest, _) = take_while(|ch: char| ch.is_ascii_whitespace())(rest)?;
        Ok((rest, output))
    }
}

pub fn str<'a>(mut input: &'a str) -> IResult<&str, Cow<'_, str>> {
    input = input
        .strip_prefix('"')
        .ok_or_else(|| NomErr::Error(NomError::new(input, NomErrorKind::Tag)))?;
    let mut chars = input.char_indices().peekable();

    let mut idx = 0;
    let mut parsed = Cow::Borrowed("");
    while let Some((ch_idx, ch)) = chars.next() {
        match ch {
            // escaped delimiter, ignore back slash and push
            '\\' if matches!(chars.peek(), Some((_, '"'))) => {
                // discard the quote
                let _ = chars.next();
                parsed.to_mut().push('"');
                idx = ch_idx + 2;
            }
            // delimiter found, break
            '"' => {
                idx = ch_idx + 1;
                break;
            }
            // push this
            ch => {
                parsed = match parsed {
                    Cow::Borrowed(_) => Cow::Borrowed(&input[..ch_idx + ch.len_utf8()]),
                    Cow::Owned(mut string) => {
                        string.push(ch);
                        Cow::Owned(string)
                    }
                };

                idx = ch_idx + ch.len_utf8();
            }
        }
    }

    Ok((&input[idx..], parsed))
}

// TODO: ignore multiple spaces
pub fn eolina_array<'a>(input: &'a str) -> IResult<&str, Vec<Cow<'_, str>>> {
    delimited(
        tag("("),
        separated_list0(ignore_ascii_whitespace(tag(",")), str),
        tag(")"),
    )(input)
}

// TODO: i think the explicit error can be removed
pub fn isize<'a>(input: &'a str) -> IResult<&str, &str> {
    let (rest, (sign, num)) = pair(opt(tag("-")), digit1)(input)?;

    Ok((
        rest,
        &input[..sign.map(|_| 1).unwrap_or_default() + num.len()],
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ignore_ascii_whitespace() {
        assert_eq!(aiw(tag(","))(",").unwrap(), ("", ","));
        assert_eq!(aiw(tag(","))("   ,").unwrap(), ("", ","));
        assert_eq!(aiw(tag(","))(",   ").unwrap(), ("", ","));
        assert_eq!(aiw(tag(","))("    ,   ").unwrap(), ("", ","));
    }

    #[test]
    fn str() {
        assert_eq!(super::str(r#""""#).unwrap(), ("", Cow::Borrowed("")));
        assert_eq!(
            super::str(r#""\"""#).unwrap(),
            ("", Cow::Owned("\"".to_owned()))
        );
        assert_eq!(
            super::str(r#""\ """#).unwrap(),
            ("\"", Cow::Borrowed("\\ "))
        );
    }

    #[test]
    fn eolina_array() {
        assert_eq!(
            super::eolina_array(r#"("")"#).unwrap(),
            ("", vec![Cow::Borrowed("")])
        );
        assert_eq!(
            super::eolina_array(r#"("", "aaa")"#).unwrap(),
            ("", vec![Cow::Borrowed(""), Cow::Borrowed("aaa")])
        );
        assert_eq!(
            super::eolina_array(r#"("\"")"#).unwrap(),
            ("", vec![Cow::Borrowed("\"")])
        );
    }

    #[test]
    fn isize() {
        assert_eq!(super::isize("0").unwrap(), ("", "0"));
        assert_eq!(super::isize("0aaa").unwrap(), ("aaa", "0"));
        assert_eq!(super::isize("-4").unwrap(), ("", "-4"));
        assert_eq!(super::isize("4aaa").unwrap(), ("aaa", "4"));
    }
}
