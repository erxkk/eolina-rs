use super::{Error, Value, ValueKind};
use crate::helper::{AsciiExt, EolinaRange, EolinaRangeBound};
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
            &[ValueKind::String, ValueKind::StringVec],
            x.kind(),
        )),
        (Value::Bool(_), x) => Err(Error::arg_mismatch(
            &[ValueKind::String, ValueKind::StringVec],
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
            &[ValueKind::String, ValueKind::StringVec],
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
            &[ValueKind::String, ValueKind::StringVec],
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
            &[ValueKind::String, ValueKind::StringVec],
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
pub fn slice(input: Value, range: EolinaRange) -> Result<Value, Error> {
    let len = input.unwrap_len()?;

    // map to absolute
    let idx_map = |idx| match idx {
        EolinaRangeBound::Start(idx) => idx as isize,
        EolinaRangeBound::End(idx) => len as isize - idx as isize,
    };

    // check if valid
    let abs_check_map = |abs| {
        if abs < 0 {
            Err(abs)
        } else if abs > len as isize {
            Err(abs)
        } else {
            Ok(abs as usize)
        }
    };

    // map to absolute unsigned
    let abs_lower = range.start.map(idx_map).map(abs_check_map).unwrap_or(Ok(0));
    let abs_upper = range.end.map(idx_map).map(abs_check_map).unwrap_or(Ok(len));

    // handle error cases
    let (lower, upper) = match (abs_lower, abs_upper) {
        (Ok(ok), Err(err)) => return Err(Error::slice_oor(range, ok as isize..err, len)),
        (Err(err), Ok(ok)) => return Err(Error::slice_oor(range, err..ok as isize, len)),
        (Err(err_l), Err(err_u)) => return Err(Error::slice_oor(range, err_l..err_u, len)),
        (Ok(ok_l), Ok(ok_u)) => {
            if ok_l > ok_u {
                return Err(Error::slice_incompat(
                    range,
                    ok_l as isize..ok_u as isize,
                    len,
                ));
            } else {
                (ok_l, ok_u)
            }
        }
    };

    match input {
        Value::String(string) => Ok(Value::String(string[lower..upper].to_owned())),
        Value::StringVec(vec) => Ok(Value::StringVec(vec[lower..upper].to_owned())),
        x => Err(Error::arg_mismatch(
            &[ValueKind::String, ValueKind::StringVec],
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
        assert_eq!(
            super::split(Value::Bool(true)).unwrap_err(),
            Error::arg_mismatch(&[ValueKind::String], ValueKind::Bool)
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
        assert_eq!(
            super::join(Value::Bool(true)).unwrap_err(),
            Error::arg_mismatch(&[ValueKind::StringVec], ValueKind::Bool)
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
        assert_eq!(
            super::is_conso(Value::Bool(true)).unwrap_err(),
            Error::arg_mismatch(&[ValueKind::String, ValueKind::StringVec], ValueKind::Bool)
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
        assert_eq!(
            super::is_vowel(Value::Bool(true)).unwrap_err(),
            Error::arg_mismatch(&[ValueKind::String, ValueKind::StringVec], ValueKind::Bool)
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
        assert_eq!(
            super::is_lower(Value::Bool(true)).unwrap_err(),
            Error::arg_mismatch(&[ValueKind::String, ValueKind::StringVec], ValueKind::Bool)
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
        assert_eq!(
            super::is_upper(Value::Bool(true)).unwrap_err(),
            Error::arg_mismatch(&[ValueKind::String, ValueKind::StringVec], ValueKind::Bool)
        );
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
        assert_eq!(
            super::map(Value::Bool(true), MapToken::Lower).unwrap_err(),
            Error::arg_mismatch(&[ValueKind::String, ValueKind::StringVec], ValueKind::Bool)
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
        assert_eq!(
            super::filter(Value::Bool(true), CheckToken::Vowel).unwrap_err(),
            Error::arg_mismatch(&[ValueKind::String, ValueKind::StringVec], ValueKind::Bool)
        );
    }

    #[test]
    fn slice_pos() {
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (..).into()).unwrap(),
            Value::String("abcdefg".to_owned())
        );
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (3..).into()).unwrap(),
            Value::String("defg".to_owned())
        );
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (..3).into()).unwrap(),
            Value::String("abc".to_owned())
        );

        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (3..2).into()).unwrap_err(),
            Error::slice_incompat(3..2, 3..2, 7)
        );
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (8..).into()).unwrap_err(),
            Error::slice_oor(8.., 8..7, 7)
        );
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (..8).into()).unwrap_err(),
            Error::slice_oor(..8, 0..8, 7)
        );
    }

    #[test]
    fn slice_neg() {
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (-3..).into()).unwrap(),
            Value::String("efg".to_owned())
        );
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (..-3).into()).unwrap(),
            Value::String("abcd".to_owned())
        );

        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (-2..-3).into()).unwrap_err(),
            Error::slice_incompat(-2..-3, 5..4, 7)
        );
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (-8..).into()).unwrap_err(),
            Error::slice_oor(-8.., -1..7, 7)
        );
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (..-8).into()).unwrap_err(),
            Error::slice_oor(..-8, 0..-1, 7)
        );
    }

    #[test]
    fn slice_neg_pos() {
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (-3..7).into()).unwrap(),
            Value::String("efg".to_owned())
        );
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (3..-3).into()).unwrap(),
            Value::String("d".to_owned())
        );
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (-8..3).into()).unwrap_err(),
            Error::slice_oor(-8..3, -1..3, 7)
        );
        assert_eq!(
            super::slice(Value::String("abcdefg".to_owned()), (3..-8).into()).unwrap_err(),
            Error::slice_oor(3..-8, 3..-1, 7)
        );
    }
}
