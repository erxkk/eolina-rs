use super::{ArgMismatchError, Error, Kind, Value};
use crate::helper::{AsciiExt, EolinaRange};
use crate::parse::{CheckToken, MapToken};

///
/// Splits the given input into it's [`char`]s if no `split` is given otherwise splits by `split`.
///
/// ### Accepts
///
/// * [`Kind::String`]
///
/// ### Returns
///
/// * [`Ok(Value::StringVec(vec))`]
///   * `vec` contains the input's [`char`]s, each as a separate [`String`]
/// * [`Err(error)`]
///   * `error` contains an arg type mismatch [`Error`]
///
pub fn split(input: Value, split: Option<&str>) -> Result<Value, Error> {
    let string = match input {
        Value::String(inner) => Ok(inner),
        x => Err(Error::ArgMismatch(ArgMismatchError::new(
            &[Kind::String],
            x.kind(),
        ))),
    }?;

    Ok(Value::StringVec(match split {
        Some(split) => string
            .split(&split)
            .filter(|str| !str.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
        None => string.chars().map(|ch| ch.to_string()).collect(),
    }))
}

///
/// Joins the given input into one [`String`].
///
/// ### Accepts
///
/// * [`Kind::StringVec`]
///
/// ### Returns
///
/// * [`Ok(Value::String(string))`]
///   * `string` contains the input's [`char`]s, each as a separate [`String`]
/// * [`Err(error)`]
///   * `error` contains an arg type mismatch [`Error`]
///
pub fn join(input: Value) -> Result<Value, Error> {
    let vec = match input {
        Value::StringVec(inner) => Ok(inner),
        x => Err(Error::ArgMismatch(ArgMismatchError::new(
            &[Kind::StringVec],
            x.kind(),
        ))),
    }?;

    Ok(Value::String(vec.into_iter().collect::<String>()))
}

///
/// Concatenates the given inputs into one of the same type.
///
/// ### Accepts
///
/// * [`Kind::String`]
/// * [`Kind::StringVec`]
///
/// ### Returns
///
/// * [`Ok(stringOrVec)`]
///   * `stringOrVec` contains concatenated input
/// * [`Err(error)`]
///   * `error` contains an arg type mismatch or type mismatch [`Error`]
///
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
        (x, Value::Bool(_)) => Err(Error::ArgMismatch(ArgMismatchError::new(
            &[Kind::String, Kind::StringVec],
            x.kind(),
        ))),
        (Value::Bool(_), x) => Err(Error::ArgMismatch(ArgMismatchError::new(
            &[Kind::String, Kind::StringVec],
            x.kind(),
        ))),
        x => Err(Error::Mismatch(x.0.kind(), x.1.kind())),
    }
}

///
/// Returns whether or not each element in the given input is a consonant or contains itself only
/// consonants.
///
/// ### Accepts
///
/// * [`Kind::String`]
/// * [`Kind::StringVec`]
///
/// ### Returns
///
/// * [`Ok(Value::Bool(value))`]
///   * `value` contains whether or not the check succeeded
/// * [`Err(error)`]
///   * `error` contains an arg type mismatch [`Error`]
///
pub fn is_conso(input: Value) -> Result<Value, Error> {
    __check_all(input, |s| s.is_conso())
}

///
/// Returns whether or not each element in the given input is a consonant or contains itself only
/// vowel.
///
/// ### Accepts
///
/// * [`Kind::String`]
/// * [`Kind::StringVec`]
///
/// ### Returns
///
/// * [`Ok(Value::Bool(value))`]
///   * `value` contains whether or not the check succeeded
/// * [`Err(error)`]
///   * `error` contains an arg type mismatch [`Error`]
///
pub fn is_vowel(input: Value) -> Result<Value, Error> {
    __check_all(input, |s| s.is_vowel())
}

///
/// Returns whether or not each element in the given input is a uppercase.
///
/// ### Accepts
///
/// * [`Kind::String`]
/// * [`Kind::StringVec`]
///
/// ### Returns
///
/// * [`Ok(Value::Bool(value))`]
///   * `value` contains whether or not the check succeeded
/// * [`Err(error)`]
///   * `error` contains an arg type mismatch [`Error`]
///
pub fn is_upper(input: Value) -> Result<Value, Error> {
    __check_all(input, |s| s.is_upper())
}

///
/// Returns whether or not each element in the given input is a lowercase.
///
/// ### Accepts
///
/// * [`Kind::String`]
/// * [`Kind::StringVec`]
///
/// ### Returns
///
/// * [`Ok(Value::Bool(value))`]
///   * `value` contains whether or not the check succeeded
/// * [`Err(error)`]
///   * `error` contains an arg type mismatch [`Error`]
///
pub fn is_lower(input: Value) -> Result<Value, Error> {
    __check_all(input, |s| s.is_lower())
}

fn __check_all(input: Value, check: impl Fn(&String) -> bool) -> Result<Value, Error> {
    match input {
        Value::String(string) => Ok(Value::Bool(check(&string))),
        Value::StringVec(vec) => Ok(Value::Bool(vec.into_iter().all(|string| check(&string)))),
        x => Err(Error::ArgMismatch(ArgMismatchError::new(
            &[Kind::String, Kind::StringVec],
            x.kind(),
        ))),
    }
}

///
/// Maps each element in the given input with a given map.
///
/// ### Accepts
///
/// * [`Kind::String`]
/// * [`Kind::StringVec`]
///
/// ### Returns
///
/// * [`Ok(stringOrVec)`]
///   * `stringOrVec` contains the mapped input
/// * [`Err(error)`]
///   * `error` contains an arg type mismatch [`Error`]
///
pub fn map(input: Value, map: MapToken) -> Result<Value, Error> {
    match input {
        Value::String(string) => Ok(Value::String(__map(string, map))),
        Value::StringVec(vec) => Ok(Value::StringVec(
            vec.into_iter().map(|string| __map(string, map)).collect(),
        )),
        x => Err(Error::ArgMismatch(ArgMismatchError::new(
            &[Kind::String, Kind::StringVec],
            x.kind(),
        ))),
    }
}

fn __map<T: AsciiExt>(val: T, map: MapToken) -> T {
    match map {
        MapToken::Lower => val.into_lower(),
        MapToken::Upper => val.into_upper(),
        MapToken::Swap => val.into_swap(),
    }
}

///
/// Filters out each element in the given input that does not pass a given check.
///
/// ### Accepts
///
/// * [`Kind::String`]
/// * [`Kind::StringVec`]
///
/// ### Returns
///
/// * [`Ok(stringOrVec)`]
///   * `stringOrVec` contains the filtered input
/// * [`Err(error)`]
///   * `error` contains an arg type mismatch [`Error`]
///
pub fn filter(input: Value, check: CheckToken) -> Result<Value, Error> {
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
        x => Err(Error::ArgMismatch(ArgMismatchError::new(
            &[Kind::String, Kind::StringVec],
            x.kind(),
        ))),
    }
}

fn __filter<T: AsciiExt>(val: &T, check: CheckToken) -> bool {
    match check {
        CheckToken::Vowel => val.is_vowel(),
        CheckToken::Conso => val.is_conso(),
        CheckToken::Upper => val.is_upper(),
        CheckToken::Lower => val.is_lower(),
    }
}

///
/// Slices the given input at the lower and upper bounds.
///
/// ### Accepts
///
/// * [`Kind::String`]
/// * [`Kind::StringVec`]
///
/// ### Returns
///
/// * [`Ok(stringOrVec)`]
///   * `stringOrVec` contains the subslice of the input
/// * [`Err(error)`]
///   * `error` contains an arg type mismatch [`Error`]
///
pub fn slice(input: Value, range: EolinaRange) -> Result<Value, Error> {
    let len = match &input {
        Value::String(inner) => Ok(inner.len()),
        Value::StringVec(inner) => Ok(inner.len()),
        x => Err(Error::ArgMismatch(ArgMismatchError::new(
            &[Kind::String],
            x.kind(),
        ))),
    }?;

    let range = range.as_range(len)?;

    match input {
        Value::String(string) => Ok(Value::String(string[range].to_owned())),
        Value::StringVec(vec) => Ok(Value::StringVec(vec[range].to_owned())),
        x => Err(Error::ArgMismatch(ArgMismatchError::new(
            &[Kind::String, Kind::StringVec],
            x.kind(),
        ))),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn split() {
        assert_eq!(
            super::split(Value::String("Abc".to_owned()), None).unwrap(),
            Value::StringVec(vec!["A".to_owned(), "b".to_owned(), "c".to_owned()])
        );
        assert_eq!(
            super::split(Value::String("Abcbdebfb".to_owned()), Some("b")).unwrap(),
            Value::StringVec(vec![
                "A".to_owned(),
                "c".to_owned(),
                "de".to_owned(),
                "f".to_owned()
            ])
        );
        assert!(super::split(Value::Bool(true), None).is_err());
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
        assert!(super::join(Value::Bool(true)).is_err());
    }

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
        assert!(super::is_conso(Value::Bool(true)).is_err());
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
        assert!(super::is_vowel(Value::Bool(true)).is_err());
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
        assert!(super::is_lower(Value::Bool(true)).is_err());
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
        assert!(super::is_upper(Value::Bool(true)).is_err());
    }

    #[test]
    fn map() {
        assert_eq!(
            super::map(Value::String("aBc".to_owned()), MapToken::Lower).unwrap(),
            Value::String("abc".to_owned())
        );
        assert_eq!(
            super::map(Value::String("abC".to_owned()), MapToken::Upper).unwrap(),
            Value::String("ABC".to_owned())
        );
        assert_eq!(
            super::map(
                Value::StringVec(vec!["AbC".to_owned(), "dEf".to_owned()]),
                MapToken::Swap
            )
            .unwrap(),
            Value::StringVec(vec!["aBc".to_owned(), "DeF".to_owned()])
        );
        assert!(super::map(Value::Bool(true), MapToken::Lower).is_err());
    }

    #[test]
    fn filter() {
        assert_eq!(
            super::filter(Value::String("abC".to_owned()), CheckToken::Vowel).unwrap(),
            Value::String("a".to_owned())
        );
        assert_eq!(
            super::filter(Value::String("aBc".to_owned()), CheckToken::Conso).unwrap(),
            Value::String("Bc".to_owned())
        );
        assert_eq!(
            super::filter(
                Value::StringVec(vec!["ABC".to_owned(), "def".to_owned()]),
                CheckToken::Upper
            )
            .unwrap(),
            Value::StringVec(vec!["ABC".to_owned()])
        );
        assert_eq!(
            super::filter(
                Value::StringVec(vec!["abc".to_owned(), "DEF".to_owned()]),
                CheckToken::Lower
            )
            .unwrap(),
            Value::StringVec(vec!["abc".to_owned()])
        );
        assert!(super::filter(Value::Bool(true), CheckToken::Vowel).is_err());
    }

    #[test]
    fn slice_pos() {
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (..).into()).unwrap(),
            Value::String("abcdefg".to_owned())
        );
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (3usize..).into()).unwrap(),
            Value::String("defg".to_owned())
        );
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (..3usize).into()).unwrap(),
            Value::String("abc".to_owned())
        );

        assert!(super::slice(Value::String("abcdefg".to_owned()), (3..2usize).into()).is_err());
        assert!(super::slice(Value::String("abcdefg".to_owned()), (8usize..).into()).is_err());
        assert!(super::slice(Value::String("abcdefg".to_owned()), (..8usize).into()).is_err());
    }

    #[test]
    fn slice_neg() {
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (-3isize..).into()).unwrap(),
            Value::String("efg".to_owned())
        );
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (..-3isize).into()).unwrap(),
            Value::String("abcd".to_owned())
        );

        assert!(super::slice(Value::String("abcdefg".to_owned()), (-2..-3isize).into()).is_err());
        assert!(super::slice(Value::String("abcdefg".to_owned()), (-8isize..).into()).is_err());
        assert!(super::slice(Value::String("abcdefg".to_owned()), (..-8isize).into()).is_err());
    }

    #[test]
    fn slice_neg_pos() {
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (-3..7isize).into()).unwrap(),
            Value::String("efg".to_owned())
        );
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (3..-3isize).into()).unwrap(),
            Value::String("d".to_owned())
        );
        assert!(super::slice(Value::String("abcdefg".to_owned()), (-8..3isize).into()).is_err());
        assert!(super::slice(Value::String("abcdefg".to_owned()), (3..-8isize).into()).is_err());
    }
}
