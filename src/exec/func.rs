use super::{Error, Value, ValueKind};
use crate::helper::AsciiExt;
use crate::parse::{CheckToken, MapToken};

///
/// Splits the given input into it's [`char`]s.
///
/// ### Accepts
///
/// * [`ValueKind::String`]
///
/// ### Returns
///
/// * [`Ok(Value::StringVec(vec))`]
///   * `vec` contains the input's [`char`]s, each as a separate [`String`]
/// * [`Err(error)`]
///   * `error` contains an arg type mismatch [`Error`]
///
pub fn split(input: Value) -> Result<Value, Error> {
    Ok(Value::StringVec(
        input
            .unwrap_string()?
            .chars()
            .map(|ch| ch.to_string())
            .collect(),
    ))
}

///
/// Joins the given input into one [`String`].
///
/// ### Accepts
///
/// * [`ValueKind::StringVec`]
///
/// ### Returns
///
/// * [`Ok(Value::String(string))`]
///   * `string` contains the input's [`char`]s, each as a separate [`String`]
/// * [`Err(error)`]
///   * `error` contains an arg type mismatch [`Error`]
///
pub fn join(input: Value) -> Result<Value, Error> {
    Ok(Value::String(
        input.unwrap_string_vec()?.into_iter().collect::<String>(),
    ))
}

///
/// Concatenates the given inputs into one of the same type.
///
/// ### Accepts
///
/// * [`ValueKind::String`]
/// * [`ValueKind::StringVec`]
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

///
/// Returns whether or not each element in the given input is a consonant or contains itself only consonants.
///
/// ### Accepts
///
/// * [`ValueKind::String`]
/// * [`ValueKind::StringVec`]
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
/// Returns whether or not each element in the given input is a consonant or contains itself only vowel.
///
/// ### Accepts
///
/// * [`ValueKind::String`]
/// * [`ValueKind::StringVec`]
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
/// * [`ValueKind::String`]
/// * [`ValueKind::StringVec`]
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
/// * [`ValueKind::String`]
/// * [`ValueKind::StringVec`]
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
        x => Err(Error::arg_mismatch(
            &[ValueKind::StringVec, ValueKind::String],
            x.kind(),
        )),
    }
}

///
/// Maps each element in the given input with a given map.
///
/// ### Accepts
///
/// * [`ValueKind::String`]
/// * [`ValueKind::StringVec`]
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
        x => Err(Error::arg_mismatch(
            &[ValueKind::StringVec, ValueKind::String],
            x.kind(),
        )),
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
/// * [`ValueKind::String`]
/// * [`ValueKind::StringVec`]
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
        x => Err(Error::arg_mismatch(
            &[ValueKind::StringVec, ValueKind::String],
            x.kind(),
        )),
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
/// * [`ValueKind::String`]
/// * [`ValueKind::StringVec`]
///
/// ### Returns
///
/// * [`Ok(stringOrVec)`]
///   * `stringOrVec` contains the subslice of the input
/// * [`Err(error)`]
///   * `error` contains an arg type mismatch [`Error`]
///
pub fn slice(input: Value, lower: Option<isize>, upper: Option<isize>) -> Result<Value, Error> {
    let len = input.unwrap_len()?;
    let lower = lower.map(|num| if num.is_negative() { len as isize + num } else { num } as usize);
    let upper = upper.map(|num| if num.is_negative() { len as isize + num } else { num } as usize);

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
    fn map() {
        assert_eq!(
            super::map(Value::String("abC".to_owned()), MapToken::Upper).unwrap(),
            Value::String("ABC".to_owned())
        );
        assert_eq!(
            super::map(Value::String("aBc".to_owned()), MapToken::Lower).unwrap(),
            Value::String("abc".to_owned())
        );
        assert_eq!(
            super::map(
                Value::StringVec(vec!["AbC".to_owned(), "dEf".to_owned()]),
                MapToken::Swap
            )
            .unwrap(),
            Value::StringVec(vec!["aBc".to_owned(), "DeF".to_owned()])
        );
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
    }

    #[test]
    fn slice() {
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), None, None).unwrap(),
            Value::String("abcdefg".to_owned())
        );
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), Some(3), None).unwrap(),
            Value::String("defg".to_owned())
        );
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), None, Some(3)).unwrap(),
            Value::String("abc".to_owned())
        );

        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), Some(-3), None).unwrap(),
            Value::String("efg".to_owned())
        );
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), None, Some(-3)).unwrap(),
            Value::String("abcd".to_owned())
        );
    }
}
