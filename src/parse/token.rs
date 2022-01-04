use crate::{
    helper::{self, nom::OneOf2, EolinaRange},
    token::{Inline, Map, Predicate, Program, Repeat, RepeatKind, Token, Transform},
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit0,
    combinator::opt,
    error::{Error as NomError, ErrorKind},
    sequence::{delimited, pair, preceded, separated_pair},
    Err as NomErr, IResult,
};

fn repeat(input: &str) -> IResult<&str, Token<'_>> {
    let (rest, (token, count)) = pair(
        alt((tag("<"), tag(">"), tag("~"), tag("*"), tag("@"))),
        digit0,
    )(input)?;

    Ok((
        rest,
        Token::Repeat(Repeat::new(
            match token {
                "<" => RepeatKind::In,
                ">" => RepeatKind::Out,
                "~" => RepeatKind::Concat,
                "*" => RepeatKind::Duplicate,
                "@" => RepeatKind::Rotate,
                _ => unreachable!(),
            },
            {
                let parsed = count.parse().unwrap_or(1);
                if parsed == 0 {
                    1
                } else {
                    parsed
                }
            },
        )),
    ))
}

fn inline(input: &str) -> IResult<&str, Token<'_>> {
    let inner = |input| {
        let mut rest = input;
        loop {
            match try_next_token(rest) {
                Ok((rem, _)) => {
                    let trimmed = rem.trim_matches(|ch: char| ch.is_ascii_whitespace());

                    if let Some(trimed_rem) = trimmed.strip_prefix('}') {
                        break Ok((rem, &input[..input.len() - trimed_rem.len() - 1]));
                    } else {
                        rest = rem;
                    }
                }
                // the error kind given here is not used
                Err(unknown) => break Err(NomErr::Error(NomError::new(unknown, ErrorKind::Alpha))),
            };
        }
    };

    let (rest, tokens) = delimited(tag("{"), inner, tag("}"))(input)?;

    Ok((rest, Token::Inline(Inline::Program(Program::new(tokens)))))
}

// TODO: implement blocks where applicable
// block: predicate, join, split
fn transform_predicate(input: &str) -> IResult<&str, Token<'_>> {
    let (rest, predicate) = helper::nom::one_of_2(
        preceded(tag("p"), delimited(tag("["), helper::nom::str, tag("]"))),
        alt((tag("v"), tag("c"), tag("_"), tag("^"))),
    )(input)?;

    Ok((
        rest,
        match predicate {
            OneOf2::A(string) => Token::Transform(Transform::Predicate(Predicate::Contains(
                Inline::StringLiteral(string),
            ))),
            OneOf2::B("v") => Token::Transform(Transform::Predicate(Predicate::Vowel)),
            OneOf2::B("c") => Token::Transform(Transform::Predicate(Predicate::Conso)),
            OneOf2::B("_") => Token::Transform(Transform::Predicate(Predicate::Lower)),
            OneOf2::B("^") => Token::Transform(Transform::Predicate(Predicate::Upper)),
            _ => unreachable!(),
        },
    ))
}

fn transform_filter(input: &str) -> IResult<&str, Token<'_>> {
    let (rest, (_, filter)) =
        pair(tag("f"), delimited(tag("["), transform_predicate, tag("]")))(input)?;

    Ok((
        rest,
        Token::Transform(match filter {
            Token::Transform(Transform::Predicate(pred)) => Transform::Filter(pred),
            _ => unreachable!(),
        }),
    ))
}

fn transform_map(input: &str) -> IResult<&str, Token<'_>> {
    let (rest, (_, map)) = pair(
        tag("m"),
        delimited(tag("["), alt((tag("_"), tag("^"), tag("%"))), tag("]")),
    )(input)?;

    Ok((
        rest,
        match map {
            "_" => Token::Transform(Transform::Map(Map::Lower)),
            "^" => Token::Transform(Transform::Map(Map::Upper)),
            "%" => Token::Transform(Transform::Map(Map::Swap)),
            _ => unreachable!(),
        },
    ))
}

fn transform_join(input: &str) -> IResult<&str, Token<'_>> {
    let (rest, (_, join)) = pair(
        tag("."),
        opt(delimited(tag("["), helper::nom::str, tag("]"))),
    )(input)?;

    Ok((
        rest,
        Token::Transform(Transform::Join(
            join.map(|join| Inline::StringLiteral(join)),
        )),
    ))
}

fn transform_split(input: &str) -> IResult<&str, Token<'_>> {
    let (rest, (_, split)) = pair(
        tag("/"),
        opt(delimited(tag("["), helper::nom::str, tag("]"))),
    )(input)?;

    Ok((
        rest,
        Token::Transform(Transform::Split(
            split.map(|split| Inline::StringLiteral(split)),
        )),
    ))
}

fn transform(input: &str) -> IResult<&str, Token<'_>> {
    alt((
        transform_predicate,
        transform_filter,
        transform_map,
        transform_join,
        transform_split,
    ))(input)
}

fn index(input: &str) -> IResult<&str, Token<'_>> {
    let (rest, num) = delimited(tag("|"), helper::nom::isize, tag("|"))(input)?;

    Ok((rest, Token::Index(num.parse::<isize>().unwrap().into())))
}

fn slice(input: &str) -> IResult<&str, Token<'_>> {
    let (rest, (start, end)) = delimited(
        tag("|"),
        separated_pair(opt(helper::nom::isize), tag("."), opt(helper::nom::isize)),
        tag("|"),
    )(input)?;

    Ok((
        rest,
        Token::Slice(EolinaRange::new(
            start.and_then(|str: &str| str.parse::<isize>().ok()),
            end.and_then(|str: &str| str.parse::<isize>().ok()),
        )),
    ))
}

fn try_next_token(input: &str) -> Result<(&str, Token<'_>), &str> {
    let res = alt((repeat, inline, transform, slice, index))(input);
    if let Ok((rest, token)) = res {
        Ok((rest, token))
    } else {
        Err(input)
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
pub fn next_token(input: &str) -> color_eyre::Result<(&str, Token<'_>)> {
    // ignore leading whitespace
    let trimmed = input.trim_start_matches(|ch: char| ch.is_ascii_whitespace());

    if trimmed.is_empty() {
        color_eyre::eyre::bail!("the given program was empty");
    }

    if let Ok((rest, token)) = try_next_token(trimmed) {
        // we trimmed whitepace, but want to count towards the length of this token
        Ok((rest, token))
    } else {
        color_eyre::eyre::bail!(format!("unknown token at '{}'", trimmed));
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn inline() {
        assert_eq!(
            next_token("{<>}").unwrap().1,
            Token::Inline(Inline::Program(Program::new("<>")))
        );
        assert_eq!(
            next_token("{<>{</|.|}}").unwrap().1,
            Token::Inline(Inline::Program(Program::new("<>{</|.|}")))
        );
    }

    #[test]
    fn repeat() {
        assert_eq!(
            next_token("<").unwrap().1,
            Token::Repeat(Repeat::new(RepeatKind::In, 1))
        );
        assert_eq!(
            next_token(">2").unwrap().1,
            Token::Repeat(Repeat::new(RepeatKind::Out, 2))
        );
        assert_eq!(
            next_token("~").unwrap().1,
            Token::Repeat(Repeat::new(RepeatKind::Concat, 1))
        );
        assert_eq!(
            next_token("*2").unwrap().1,
            Token::Repeat(Repeat::new(RepeatKind::Duplicate, 2))
        );
        assert_eq!(
            next_token("@").unwrap().1,
            Token::Repeat(Repeat::new(RepeatKind::Rotate, 1))
        );
    }

    #[test]
    fn transform() {
        assert_eq!(
            next_token("f[^]").unwrap().1,
            Token::Transform(Transform::Filter(Predicate::Upper))
        );
        assert_eq!(
            next_token("m[%]").unwrap().1,
            Token::Transform(Transform::Map(Map::Swap))
        );
        assert_eq!(
            next_token("p[\" \"]").unwrap().1,
            Token::Transform(Transform::Predicate(Predicate::Contains(
                Inline::StringLiteral(Cow::Borrowed(" "))
            )))
        );
        assert_eq!(
            next_token(".[]").unwrap().1,
            Token::Transform(Transform::Join(None))
        );
        assert_eq!(
            next_token("/[\" \"]").unwrap().1,
            Token::Transform(Transform::Split(Some(Inline::StringLiteral(
                Cow::Borrowed(" ")
            ))))
        );
        assert_eq!(
            next_token(".").unwrap().1,
            Token::Transform(Transform::Join(None))
        );
        assert_eq!(
            next_token("/").unwrap().1,
            Token::Transform(Transform::Split(None))
        );
    }

    #[test]
    fn index() {
        assert_eq!(
            next_token("|42|").unwrap(),
            ("", Token::Index((42isize).into()))
        );
        assert_eq!(
            next_token("|-42|").unwrap(),
            ("", Token::Index((-42isize).into()))
        );
    }

    #[test]
    fn slice() {
        assert_eq!(next_token("|.|").unwrap(), ("", Token::Slice((..).into())));
        assert_eq!(
            next_token("|.42|").unwrap(),
            ("", Token::Slice((..42usize).into()))
        );
        assert_eq!(
            next_token("|-42.|").unwrap(),
            ("", Token::Slice((-42isize..).into()))
        );
        assert_eq!(
            next_token("|42.-42|").unwrap(),
            ("", Token::Slice((42..-42isize).into()))
        );
    }

    #[test]
    fn repeating() {
        let mut res = next_token("</|.|>").unwrap();
        assert_eq!(
            res,
            ("/|.|>", Token::Repeat(Repeat::new(RepeatKind::In, 1)))
        );

        res = next_token(res.0).unwrap();
        assert_eq!(res, ("|.|>", Token::Transform(Transform::Split(None))));

        res = next_token(res.0).unwrap();
        assert_eq!(res, (">", Token::Slice((..).into())));

        res = next_token(res.0).unwrap();
        assert_eq!(res, ("", Token::Repeat(Repeat::new(RepeatKind::Out, 1))));
    }
}
