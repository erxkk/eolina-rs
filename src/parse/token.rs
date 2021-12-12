use super::Error;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit0,
    error::Error as NomError,
    sequence::{delimited, separated_pair},
    Err,
};
use std::fmt::Display;

///
/// A filter token, a token between `[` and `]`.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Filter {
    Vowel,
    Conso,
    Upper,
    Lower,
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vowel => f.write_str("v"),
            Self::Conso => f.write_str("c"),
            Self::Upper => f.write_str("^"),
            Self::Lower => f.write_str("_"),
        }
    }
}

///
/// A function token.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token {
    In,
    Out,
    Split,
    Join,
    Concat,
    Copy,
    Vowel,
    Conso,
    ToUpper,
    ToLower,
    IsUpper,
    IsLower,
    Rotate,
    Filter(Filter),
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
            Self::Filter(filter) => write!(f, "[{}]", filter),
            Self::Slice(lower, upper) => {
                write!(
                    f,
                    "|{}.{}|",
                    lower.map(|num| num.to_string()).unwrap_or_else(|| "".to_owned()),
                    upper.map(|num| num.to_string()).unwrap_or_else(|| "".to_owned())
                )
            }
        }
    }
}

///
/// Finds the next token inside the given `input` string.
///
/// ### Returns
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
        tag("l"),
        tag("u"),
        tag("@"),
        tag("^"),
        tag("_"),
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
        alt((tag("v"), tag("c"), tag("^"), tag("_"))),
        tag("]"),
    ),);

    // type inference died on me here because of the error types
    let single_res: Result<(&str, &str), Err<NomError<&str>>> = alt(single)(input);
    let single_delim_res: Result<(&str, &str), Err<NomError<&str>>> = alt(single_delim)(input);
    let double_res: Result<(&str, (&str, &str)), Err<NomError<&str>>> = alt(double)(input);

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
                "l" => Token::ToLower,
                "u" => Token::ToUpper,
                "@" => Token::Rotate,
                "^" => Token::IsUpper,
                "_" => Token::IsLower,
                _ => unreachable!(),
            },
        ))
    } else if let Ok((rest, parsed)) = single_delim_res {
        Ok((
            rest,
            match parsed {
                "v" => Token::Filter(Filter::Vowel),
                "c" => Token::Filter(Filter::Conso),
                "^" => Token::Filter(Filter::Upper),
                "_" => Token::Filter(Filter::Lower),
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
        assert_eq!(next_token("<"), Ok(("", Token::In)));
        assert_eq!(next_token(">"), Ok(("", Token::Out)));
        assert_eq!(next_token("/"), Ok(("", Token::Split)));
        assert_eq!(next_token("."), Ok(("", Token::Join)));
        assert_eq!(next_token("v"), Ok(("", Token::Vowel)));
        assert_eq!(next_token("c"), Ok(("", Token::Conso)));
        assert_eq!(next_token("l"), Ok(("", Token::ToLower)));
        assert_eq!(next_token("u"), Ok(("", Token::ToUpper)));
        assert_eq!(next_token("@"), Ok(("", Token::Rotate)));
        assert_eq!(next_token("^"), Ok(("", Token::IsUpper)));
        assert_eq!(next_token("_"), Ok(("", Token::IsLower)));
    }

    #[test]
    fn filter() {
        assert_eq!(next_token("[v]"), Ok(("", Token::Filter(Filter::Vowel))));
        assert_eq!(next_token("[c]"), Ok(("", Token::Filter(Filter::Conso))));
        assert_eq!(next_token("[^]"), Ok(("", Token::Filter(Filter::Upper))));
        assert_eq!(next_token("[_]"), Ok(("", Token::Filter(Filter::Lower))));
    }

    #[test]
    fn slice() {
        assert_eq!(next_token("|.|"), Ok(("", Token::Slice(None, None))));
        assert_eq!(next_token("|.42|"), Ok(("", Token::Slice(None, Some(42)))));
        assert_eq!(next_token("|42.|"), Ok(("", Token::Slice(Some(42), None))));
        assert_eq!(
            next_token("|42.42|"),
            Ok(("", Token::Slice(Some(42), Some(42))))
        );
    }

    #[test]
    fn repeating() {
        assert_eq!(next_token("<>/|.|"), Ok((">/|.|", Token::In)));
        assert_eq!(next_token(">/|.|"), Ok(("/|.|", Token::Out)));
        assert_eq!(next_token("/|.|"), Ok(("|.|", Token::Split)));
        assert_eq!(next_token("|.|"), Ok(("", Token::Slice(None, None))));
    }
}
