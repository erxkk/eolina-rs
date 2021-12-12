use super::{Error, Value, ValueKind};
use crate::helper::AsciiCheckExt;
use crate::parse::FilterToken;

pub fn split(input: Value) -> Result<Value, Error> {
    Ok(Value::StringVec(
        input
            .unwrap_string()?
            .chars()
            .map(|ch| ch.to_string())
            .collect(),
    ))
}

pub fn join(input: Value) -> Result<Value, Error> {
    Ok(Value::String(
        input.unwrap_string_vec()?.into_iter().collect::<String>(),
    ))
}

pub fn concat(input1: Value, input2: Value) -> Result<Value, Error> {
    match (input1, input2) {
        (Value::String(mut string1), Value::String(string2)) => Ok(Value::String({
            string1.push_str(&string2);
            string1
        })),
        (Value::StringVec(mut vec1), Value::StringVec(mut vec2)) => Ok(Value::StringVec({
            vec1.append(&mut vec2);
            vec1
        })),
        (x, Value::Bool(_)) => Err(Error::arg_mismatch(
            &[ValueKind::StringVec, ValueKind::String],
            x.kind(),
        )),
        (Value::Bool(_), x) => Err(Error::arg_mismatch(
            &[ValueKind::StringVec, ValueKind::String],
            x.kind(),
        )),
        x => Err(Error::mismatch(x.0.kind(), x.1.kind())),
    }
}

pub fn to_upper(input: Value) -> Result<Value, Error> {
    __to(input, |s| s.to_upper())
}

pub fn to_lower(input: Value) -> Result<Value, Error> {
    __to(input, |s| s.to_lower())
}

pub fn __to(input: Value, converter: impl Fn(&String) -> String) -> Result<Value, Error> {
    match input {
        Value::String(string) => Ok(Value::String(converter(&string))),
        Value::StringVec(vec) => Ok(Value::StringVec(
            vec.into_iter().map(|string| converter(&string)).collect(),
        )),
        x => Err(Error::arg_mismatch(
            &[ValueKind::StringVec, ValueKind::String],
            x.kind(),
        )),
    }
}

pub fn is_conso(input: Value) -> Result<Value, Error> {
    __check_all(input, |s| s.is_conso())
}

pub fn is_vowel(input: Value) -> Result<Value, Error> {
    __check_all(input, |s| s.is_vowel())
}

pub fn is_upper(input: Value) -> Result<Value, Error> {
    __check_all(input, |s| s.is_upper())
}

pub fn is_lower(input: Value) -> Result<Value, Error> {
    __check_all(input, |s| s.is_lower())
}

fn __check_all(input: Value, check: impl Fn(&String) -> bool) -> Result<Value, Error> {
    match input {
        Value::String(string) => Ok(Value::Bool(check(&string))),
        Value::StringVec(vec) => Ok(Value::Bool(vec.into_iter().all(|string| check(&string)))),
        x => Err(Error::arg_mismatch(
            &[ValueKind::StringVec, ValueKind::String],
            x.kind(),
        )),
    }
}

pub fn filter(input: Value, check: FilterToken) -> Result<Value, Error> {
    match input {
        Value::String(string) => Ok(Value::String(
            string
                .chars()
                .filter(|ch| __filter(ch, check))
                .collect::<String>(),
        )),
        Value::StringVec(vec) => Ok(Value::StringVec(
            vec.into_iter()
                .filter(|string| __filter(string, check))
                .collect(),
        )),
        x => Err(Error::arg_mismatch(
            &[ValueKind::StringVec, ValueKind::String],
            x.kind(),
        )),
    }
}

fn __filter<T: AsciiCheckExt>(val: &T, check: FilterToken) -> bool {
    match check {
        FilterToken::Vowel => val.is_vowel(),
        FilterToken::Conso => val.is_conso(),
        FilterToken::Upper => val.is_upper(),
        FilterToken::Lower => val.is_lower(),
    }
}

pub fn slice(input: Value, lower: Option<usize>, upper: Option<usize>) -> Result<Value, Error> {
    match input {
        Value::String(string) => Ok(Value::String(match (lower, upper) {
            (Some(l), Some(u)) => string[l..u].to_owned(),
            (Some(l), None) => string[l..].to_owned(),
            (None, Some(u)) => string[..u].to_owned(),
            _ => string,
        })),
        Value::StringVec(vec) => Ok(Value::StringVec(match (lower, upper) {
            (Some(l), Some(u)) => vec[l..u].to_owned(),
            (Some(l), None) => vec[l..].to_owned(),
            (None, Some(u)) => vec[..u].to_owned(),
            _ => vec,
        })),
        x => Err(Error::arg_mismatch(
            &[ValueKind::StringVec, ValueKind::String],
            x.kind(),
        )),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn split() {
        assert_eq!(
            super::split(Value::String("Abc".to_owned())).unwrap(),
            Value::StringVec(vec!["A".to_owned(), "b".to_owned(), "c".to_owned()])
        );
    }

    #[test]
    fn join() {
        assert_eq!(
            super::join(Value::StringVec(vec![
                "A".to_owned(),
                "b".to_owned(),
                "c".to_owned()
            ]))
            .unwrap(),
            Value::String("Abc".to_owned())
        );
    }

    #[test]
    fn to_upper() {
        assert_eq!(
            super::to_upper(Value::String("Abc".to_owned())).unwrap(),
            Value::String("ABC".to_owned())
        )
    }

    #[test]
    fn to_lower() {
        assert_eq!(
            super::to_lower(Value::String("Abc".to_owned())).unwrap(),
            Value::String("abc".to_owned())
        )
    }

    #[test]
    #[test]
    fn is_conso() {
        assert_eq!(
            super::is_conso(Value::String("bcd".to_owned())).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            super::is_conso(Value::String("aei".to_owned())).unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn is_vowel() {
        assert_eq!(
            super::is_vowel(Value::String("bcd".to_owned())).unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            super::is_vowel(Value::String("aei".to_owned())).unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn is_lower() {
        assert_eq!(
            super::is_lower(Value::String("ABC".to_owned())).unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            super::is_lower(Value::String("abc".to_owned())).unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn is_upper() {
        assert_eq!(
            super::is_upper(Value::String("ABC".to_owned())).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            super::is_upper(Value::String("abc".to_owned())).unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn filter() {
        assert_eq!(
            super::filter(Value::String("abC".to_owned()), FilterToken::Vowel).unwrap(),
            Value::String("a".to_owned())
        );
        assert_eq!(
            super::filter(Value::String("aBc".to_owned()), FilterToken::Conso).unwrap(),
            Value::String("Bc".to_owned())
        );
        assert_eq!(
            super::filter(
                Value::StringVec(vec!["ABC".to_owned(), "def".to_owned()]),
                FilterToken::Upper
            )
            .unwrap(),
            Value::StringVec(vec!["ABC".to_owned()])
        );
        assert_eq!(
            super::filter(
                Value::StringVec(vec!["abc".to_owned(), "DEF".to_owned()]),
                FilterToken::Lower
            )
            .unwrap(),
            Value::StringVec(vec!["abc".to_owned()])
        );
    }
}
