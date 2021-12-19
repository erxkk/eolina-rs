use std::{borrow::Cow, fmt::Display, iter};

///
/// Formats an [`Iterator`] with the underlying [`Iterator::Item`]'s
/// [`Display`] representation.
///
pub fn fmt_iter<T: Display>(iter: impl Iterator<Item = T>) -> String {
    iter::once(Cow::Borrowed("["))
        .chain(
            iter.map(|entry| Cow::Owned(entry.to_string()))
                .intersperse(Cow::Borrowed(", ")),
        )
        .chain(iter::once(Cow::Borrowed("]")))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::program::Value;

    #[test]
    fn fmt_iter_empty() {
        let res = fmt_iter(std::iter::empty::<String>());
        assert_eq!(res, "[]")
    }

    #[test]
    fn fmt_iter_string() {
        let res = fmt_iter(["aaa", "bbb", "ccc"].into_iter());
        assert_eq!(res, "[aaa, bbb, ccc]")
    }

    #[test]
    fn fmt_iter_value() {
        let res = fmt_iter(
            [
                Value::String("aaa".to_owned()),
                Value::String("bbb".to_owned()),
                Value::String("ccc".to_owned()),
            ]
            .into_iter(),
        );
        assert_eq!(res, r#"["aaa", "bbb", "ccc"]"#)
    }
}
