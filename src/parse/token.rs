use super::Error;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit0,
    error::Error as NomError,
    sequence::{delimited, separated_pair},
    Err as NomErr,
};
use std::fmt::Display;

///
/// A filter token, a token between `[` and `]`.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Filter {
    ///
    /// The vowel filter `v`.
    ///
    Vowel,

    ///
    /// The vowel filter `c`.
    ///
    Conso,

    ///
    /// The uppercase filter `u`.
    ///
    Upper,

    ///
    /// The lowercase filter `l`.
    ///
    Lower,
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vowel => f.write_str("[v]"),
            Self::Conso => f.write_str("[c]"),
            Self::Upper => f.write_str("[^]"),
            Self::Lower => f.write_str("[_]"),
        }
    }
}

///
/// A function token.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token {
    ///
    /// The input token `<`.
    ///
    In,

    ///
    /// The output token `>`.
    ///
    Out,

    ///
    /// The split token `/`.
    ///
    Split,

    ///
    /// The join token `.`.
    ///
    Join,

    ///
    /// The concatenation token `~`.
    ///
    Concat,

    ///
    /// The copy token `*`.
    ///
    Copy,

    ///
    /// The vowel check token `v`.
    ///
    Vowel,

    ///
    /// The consonant check token `c`.
    ///
    Conso,

    ///
    /// The lowercase tcheck oken `_`.
    ///
    IsLower,

    ///
    /// The uppercase check token `^`.
    ///
    IsUpper,

    ///
    /// The lower token `l`.
    ///
    ToLower,

    ///
    /// The upper token `u`.
    ///
    ToUpper,

    ///
    /// The rotate token `@`.
    ///
    Rotate,

    ///
    /// The filter token `[x]` where `x` is a [`Filter`] token.
    ///
    Filter(Filter),

    ///
    /// The slice token `|x.x|` where `x` are empty or [`usize`].
    ///
    Slice(Option<usize>, Option<usize>),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::In => f.write_str("<"),
            Self::Out => f.write_str(">"),
            Self::Split => f.write_str("/"),
            Self::Join => f.write_str("."),
            Self::Concat => f.write_str("~"),
            Self::Copy => f.write_str("*"),
            Self::Vowel => f.write_str("v"),
            Self::Conso => f.write_str("c"),
            Self::ToUpper => f.write_str("u"),
            Self::ToLower => f.write_str("l"),
            Self::Rotate => f.write_str("@"),
            Self::IsUpper => f.write_str("^"),
            Self::IsLower => f.write_str("_"),
            Self::Filter(filter) => filter.fmt(f),
            Self::Slice(lower, upper) => {
                write!(
                    f,
                    "|{}.{}|",
                    lower
                        .map(|num| num.to_string())
                        .unwrap_or_else(|| "".to_owned()),
                    upper
                        .map(|num| num.to_string())
                        .unwrap_or_else(|| "".to_owned())
                )
            }
        }
    }
}

///
/// Finds the next token inside the given `input` string.
///
/// ### Returns
///
/// * [`Ok((rest, token))`] if the `input` starts with a valid token
///   * `rest` contains the the rest of the string after the parsed token
///   * `token` contains the parsed [`Token`]
/// * [`Err(error)`] if unable to parse a token
///   * `error` contains the [`Error`]
///
pub fn next_token(input: &str) -> Result<(&str, Token), Error> {
    let input = input.trim_start_matches(|ch| ch == ' ' || ch == '\n' || ch == '\t');
    if input.is_empty() {
        return Err(Error::empty());
    }

    // TODO: skip whitespace

    let single = (
        tag("<"),
        tag(">"),
        tag("/"),
        tag("."),
        tag("~"),
        tag("*"),
        tag("v"),
        tag("c"),
        tag("_"),
        tag("^"),
        tag("l"),
        tag("u"),
        tag("@"),
    );

    let double = (delimited(
        tag("|"),
        separated_pair(digit0, tag("."), digit0),
        tag("|"),
    ),);

    // the reuse of of regular tokens for filters causes ambiguity and does not allow
    // for those to be included in the `single` alt call
    let single_delim = (delimited(
        tag("["),
        alt((tag("v"), tag("c"), tag("_"), tag("^"))),
        tag("]"),
    ),);

    // type inference died on me here because of the error types
    let single_res: Result<(&str, &str), NomErr<NomError<&str>>> = alt(single)(input);
    let single_delim_res: Result<(&str, &str), NomErr<NomError<&str>>> = alt(single_delim)(input);
    let double_res: Result<(&str, (&str, &str)), NomErr<NomError<&str>>> = alt(double)(input);

    if let Ok((rest, parsed)) = single_res {
        Ok((
            rest,
            match parsed {
                "<" => Token::In,
                ">" => Token::Out,
                "/" => Token::Split,
                "." => Token::Join,
                "~" => Token::Concat,
                "*" => Token::Copy,
                "v" => Token::Vowel,
                "c" => Token::Conso,
                "_" => Token::IsLower,
                "^" => Token::IsUpper,
                "l" => Token::ToLower,
                "u" => Token::ToUpper,
                "@" => Token::Rotate,
                _ => unreachable!(),
            },
        ))
    } else if let Ok((rest, parsed)) = single_delim_res {
        Ok((
            rest,
            match parsed {
                "v" => Token::Filter(Filter::Vowel),
                "c" => Token::Filter(Filter::Conso),
                "_" => Token::Filter(Filter::Lower),
                "^" => Token::Filter(Filter::Upper),
                _ => unreachable!(),
            },
        ))
    } else if let Ok((rest, (first, second))) = double_res {
        Ok((rest, Token::Slice(first.parse().ok(), second.parse().ok())))
    } else {
        Err(Error::unknown(input.to_owned()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn slash() {
        assert_eq!(next_token("<").unwrap(), ("", Token::In));
        assert_eq!(next_token(">").unwrap(), ("", Token::Out));
        assert_eq!(next_token("/").unwrap(), ("", Token::Split));
        assert_eq!(next_token(".").unwrap(), ("", Token::Join));
        assert_eq!(next_token("v").unwrap(), ("", Token::Vowel));
        assert_eq!(next_token("c").unwrap(), ("", Token::Conso));
        assert_eq!(next_token("l").unwrap(), ("", Token::ToLower));
        assert_eq!(next_token("u").unwrap(), ("", Token::ToUpper));
        assert_eq!(next_token("_").unwrap(), ("", Token::IsLower));
        assert_eq!(next_token("^").unwrap(), ("", Token::IsUpper));
        assert_eq!(next_token("@").unwrap(), ("", Token::Rotate));
    }

    #[test]
    fn filter() {
        assert_eq!(
            next_token("[v]").unwrap(),
            ("", Token::Filter(Filter::Vowel))
        );
        assert_eq!(
            next_token("[c]").unwrap(),
            ("", Token::Filter(Filter::Conso))
        );
        assert_eq!(
            next_token("[_]").unwrap(),
            ("", Token::Filter(Filter::Lower))
        );
        assert_eq!(
            next_token("[^]").unwrap(),
            ("", Token::Filter(Filter::Upper))
        );
    }

    #[test]
    fn slice() {
        assert_eq!(next_token("|.|").unwrap(), ("", Token::Slice(None, None)));
        assert_eq!(
            next_token("|.42|").unwrap(),
            ("", Token::Slice(None, Some(42)))
        );
        assert_eq!(
            next_token("|42.|").unwrap(),
            ("", Token::Slice(Some(42), None))
        );
        assert_eq!(
            next_token("|42.42|").unwrap(),
            ("", Token::Slice(Some(42), Some(42)))
        );
    }

    #[test]
    fn repeating() {
        assert_eq!(next_token("<>/|.|").unwrap(), (">/|.|", Token::In));
        assert_eq!(next_token(">/|.|").unwrap(), ("/|.|", Token::Out));
        assert_eq!(next_token("/|.|").unwrap(), ("|.|", Token::Split));
        assert_eq!(next_token("|.|").unwrap(), ("", Token::Slice(None, None)));
    }
}
