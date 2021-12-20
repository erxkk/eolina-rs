use crate::helper::EolinaRange;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{digit0, digit1},
    combinator::opt,
    error::Error as NomError,
    sequence::{delimited, pair, separated_pair},
    Err as NomErr,
};
use std::fmt::{self, Display, Formatter};

///
/// A filter or map token, a token between `[` and `]`.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Check {
    ///
    /// The vowel filter `v`.
    ///
    Vowel,

    ///
    /// The vowel filter `c`.
    ///
    Conso,

    ///
    /// The lowercase filter `_`.
    ///
    Lower,

    ///
    /// The uppercase filter `^`.
    ///
    Upper,
}

impl Display for Check {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vowel => f.write_str("[v]"),
            Self::Conso => f.write_str("[c]"),
            Self::Lower => f.write_str("[_]"),
            Self::Upper => f.write_str("[^]"),
        }
    }
}

///
/// A filter or map token, a token between `{` and `}`.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Map {
    ///
    /// The lowercase map `_`.
    ///
    Lower,

    ///
    /// The uppercase map `^`.
    ///
    Upper,

    ///
    /// The swap case map `%`.
    ///
    Swap,
}

impl Display for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lower => f.write_str("{_}"),
            Self::Upper => f.write_str("{^}"),
            Self::Swap => f.write_str("{%}"),
        }
    }
}

///
/// A function token.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token<'p> {
    ///
    /// The input token `<`.
    ///
    In,

    ///
    /// The output token `>`.
    ///
    Out,

    ///
    /// The split token `/x/` where `x` is empty or a [`str`].
    ///
    Split(Option<&'p str>),

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
    IsVowel,

    ///
    /// The consonant check token `c`.
    ///
    IsConso,

    ///
    /// The lowercase tcheck oken `_`.
    ///
    IsLower,

    ///
    /// The uppercase check token `^`.
    ///
    IsUpper,

    ///
    /// The rotate token `@x` where `x` is empty or [`usize`].
    ///
    Rotate(usize),

    ///
    /// A check token `{x}` where `x` is a [`Map`] token.
    ///
    Map(Map),

    ///
    /// The filter token `[x]` where `x` is a [`Check`] token.
    ///
    Filter(Check),

    ///
    /// The slice token `|x.x|` where `x` are empty or [`isize`].
    ///
    Slice(EolinaRange),
}

impl<'p> Display for Token<'p> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::In => f.write_str("<"),
            Self::Out => f.write_str(">"),
            Self::Split(split) => match split {
                Some(string) => write!(f, "/{:?}/", string),
                None => f.write_str("//"),
            },
            Self::Join => f.write_str("."),
            Self::Concat => f.write_str("~"),
            Self::Copy => f.write_str("*"),
            Self::IsVowel => f.write_str("v"),
            Self::IsConso => f.write_str("c"),
            Self::Rotate(num) => write!(f, "@{}", num),
            Self::IsUpper => f.write_str("^"),
            Self::IsLower => f.write_str("_"),
            Self::Map(map) => map.fmt(f),
            Self::Filter(filter) => filter.fmt(f),
            Self::Slice(range) => range.fmt(f),
        }
    }
}

///
/// Finds the next token inside the given `input` string.
///
/// ### Returns
///
/// * [`Ok`]
///   * the `input` starts with a valid token, contains the unparsed rest of
///     the string after the parsed token and the parsed [`Token`]
/// * [`Err`]
///   * unable to parse a token, contains the [`Error`]
///
pub fn next_token(input: &str) -> eyre::Result<(&str, Token, usize)> {
    // ignore whitespace, treat whitespace as empty
    let trimmed = input.trim_start_matches(|ch: char| ch.is_ascii_whitespace());
    if trimmed.is_empty() {
        eyre::bail!("the given program was empty");
    }

    let tirmlen = input.len() - trimmed.len();

    let single = (
        tag("<"),
        tag(">"),
        tag("."),
        tag("~"),
        tag("*"),
        tag("v"),
        tag("c"),
        tag("_"),
        tag("^"),
    );

    let double = (pair(tag("@"), digit0),);

    let mut split = delimited(
        tag("/"),
        opt(delimited(
            tag("\""),
            take_till(|ch| matches!(ch, '"' | '/')),
            tag("\""),
        )),
        tag("/"),
    );

    let mut slice = delimited(
        tag("|"),
        separated_pair(
            opt(pair(opt(tag("-")), digit1)),
            tag("."),
            opt(pair(opt(tag("-")), digit1)),
        ),
        tag("|"),
    );

    let mut filter = delimited(
        tag("["),
        alt((tag("v"), tag("c"), tag("_"), tag("^"))),
        tag("]"),
    );

    let mut map = delimited(tag("{"), alt((tag("_"), tag("^"), tag("%"))), tag("}"));

    type Single<'a> = Result<(&'a str, &'a str), NomErr<NomError<&'a str>>>;
    type SingleOpt<'a> = Result<(&'a str, Option<&'a str>), NomErr<NomError<&'a str>>>;
    type Double<'a> = Result<(&'a str, (&'a str, &'a str)), NomErr<NomError<&'a str>>>;
    type NestedDouble<'a> = Result<
        (
            &'a str,
            (
                Option<(Option<&'a str>, &'a str)>,
                Option<(Option<&'a str>, &'a str)>,
            ),
        ),
        NomErr<NomError<&'a str>>,
    >;

    let single_res: Single = alt(single)(trimmed);
    if let Ok((rest, parsed)) = single_res {
        return Ok((
            rest,
            match parsed {
                "<" => Token::In,
                ">" => Token::Out,
                "." => Token::Join,
                "~" => Token::Concat,
                "*" => Token::Copy,
                "v" => Token::IsVowel,
                "c" => Token::IsConso,
                "_" => Token::IsLower,
                "^" => Token::IsUpper,
                _ => unimplemented!("missing single branches"),
            },
            tirmlen + 1,
        ));
    }

    let double_res: Double = alt(double)(trimmed);
    if let Ok((rest, (_, second))) = double_res {
        return Ok((
            rest,
            Token::Rotate(second.parse().unwrap_or(1)),
            tirmlen + 1 + second.len(),
        ));
    }

    let split_res: SingleOpt = split(trimmed);
    if let Ok((rest, optional)) = split_res {
        return Ok((
            rest,
            Token::Split(optional),
            tirmlen + optional.map(|str| str.len() + 4).unwrap_or(2),
        ));
    }

    let map_res: Single = map(trimmed);
    if let Ok((rest, parsed)) = map_res {
        return Ok((
            rest,
            match parsed {
                "_" => Token::Map(Map::Lower),
                "^" => Token::Map(Map::Upper),
                "%" => Token::Map(Map::Swap),
                _ => unimplemented!("missing map branches"),
            },
            tirmlen + 3,
        ));
    }

    let filter_res: Single = filter(trimmed);
    if let Ok((rest, parsed)) = filter_res {
        return Ok((
            rest,
            match parsed {
                "v" => Token::Filter(Check::Vowel),
                "c" => Token::Filter(Check::Conso),
                "_" => Token::Filter(Check::Lower),
                "^" => Token::Filter(Check::Upper),
                _ => unimplemented!("missing filter branches"),
            },
            tirmlen + 3,
        ));
    }

    let slice_res: NestedDouble = slice(trimmed);
    if let Ok((rest, (first, second))) = slice_res {
        let parser = |(sign, num): (Option<&str>, &str)| {
            (
                sign.is_some(),
                num.parse().expect("combinator must not fail"),
            )
        };

        let counter =
            |(sign, num): (Option<&str>, &str)| num.len() + if sign.is_some() { 1 } else { 0 };

        return Ok((
            rest,
            Token::Slice(EolinaRange::components(
                first.map(parser),
                second.map(parser),
            )),
            tirmlen
                + 3
                + first.map(counter).unwrap_or_default()
                + second.map(counter).unwrap_or_default(),
        ));
    }

    eyre::bail!(format!("unknown token at '{}'", input));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn single() {
        assert_eq!(next_token("<").unwrap(), ("", Token::In, 1));
        assert_eq!(next_token(">").unwrap(), ("", Token::Out, 1));
        assert_eq!(next_token(".").unwrap(), ("", Token::Join, 1));
        assert_eq!(next_token("v").unwrap(), ("", Token::IsVowel, 1));
        assert_eq!(next_token("c").unwrap(), ("", Token::IsConso, 1));
        assert_eq!(next_token("_").unwrap(), ("", Token::IsLower, 1));
        assert_eq!(next_token("^").unwrap(), ("", Token::IsUpper, 1));
    }

    #[test]
    fn split() {
        assert_eq!(next_token("//").unwrap(), ("", Token::Split(None), 2));
        assert_eq!(
            next_token("/\"\"/").unwrap(),
            ("", Token::Split(Some("")), 4)
        );
        assert_eq!(
            next_token("/\"aa\"/").unwrap(),
            ("", Token::Split(Some("aa")), 6)
        );
    }

    #[test]
    fn dobule() {
        assert_eq!(next_token("@").unwrap(), ("", Token::Rotate(1), 1));
        assert_eq!(next_token("@1").unwrap(), ("", Token::Rotate(1), 2));
        assert_eq!(next_token("@3").unwrap(), ("", Token::Rotate(3), 2));
    }

    #[test]
    fn map() {
        assert_eq!(next_token("{_}").unwrap(), ("", Token::Map(Map::Lower), 3));
        assert_eq!(next_token("{^}").unwrap(), ("", Token::Map(Map::Upper), 3));
        assert_eq!(next_token("{%}").unwrap(), ("", Token::Map(Map::Swap), 3));
    }

    #[test]
    fn filter() {
        assert_eq!(
            next_token("[v]").unwrap(),
            ("", Token::Filter(Check::Vowel), 3)
        );
        assert_eq!(
            next_token("[c]").unwrap(),
            ("", Token::Filter(Check::Conso), 3)
        );
        assert_eq!(
            next_token("[_]").unwrap(),
            ("", Token::Filter(Check::Lower), 3)
        );
        assert_eq!(
            next_token("[^]").unwrap(),
            ("", Token::Filter(Check::Upper), 3)
        );
    }

    #[test]
    fn slice_pos() {
        assert_eq!(
            next_token("|.|").unwrap(),
            ("", Token::Slice((..).into()), 3)
        );
        assert_eq!(
            next_token("|.42|").unwrap(),
            ("", Token::Slice((..42usize).into()), 5)
        );
        assert_eq!(
            next_token("|42.|").unwrap(),
            ("", Token::Slice((42usize..).into()), 5)
        );
        assert_eq!(
            next_token("|42.42|").unwrap(),
            ("", Token::Slice((42..42usize).into()), 7)
        );
    }

    #[test]
    fn slice_neg() {
        assert_eq!(
            next_token("|.-42|").unwrap(),
            ("", Token::Slice((..-42isize).into()), 6)
        );
        assert_eq!(
            next_token("|-42.|").unwrap(),
            ("", Token::Slice((-42isize..).into()), 6)
        );
        assert_eq!(
            next_token("|-42.-42|").unwrap(),
            ("", Token::Slice((-42..-42isize).into()), 9)
        );
    }

    #[test]
    fn repeating() {
        assert_eq!(next_token("<>//|.|").unwrap(), (">//|.|", Token::In, 1));
        assert_eq!(next_token(">//|.|").unwrap(), ("//|.|", Token::Out, 1));
        assert_eq!(next_token("//|.|").unwrap(), ("|.|", Token::Split(None), 2));
        assert_eq!(
            next_token("|.|").unwrap(),
            ("", Token::Slice((..).into()), 3)
        );
    }
}
