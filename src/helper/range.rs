use std::{
    fmt::{self, Debug, Display, Formatter, Write},
    ops::{Range, RangeFrom, RangeFull, RangeTo},
};

#[derive(thiserror::Error, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum IndexError {
    ///
    /// The given index was out side of the allowed target range
    ///
    #[error("index out of range: rel {0} => abs {1} vs valid |0.{2}|")]
    OutOfTargetRange(EolinaIndex, isize, usize),
}

#[derive(thiserror::Error, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum RangeError {
    ///
    /// The given range bounds are incompatible after relaltive to absolute conversion. Contains
    /// the given relative range, and the computed absolute range.
    ///
    #[error("incompatible bounds: rel {0} => abs {1} (start > end)")]
    IncompatibleBounds(EolinaRange, EolinaRange),

    ///
    /// The given range bounds are out of the valid target range. Contains the given relative
    /// range, the absolute computed range and the valid range.
    ///
    #[error("slice out of range: rel {0} => abs {1} vs valid {2}")]
    OutOfTargetRange(EolinaRange, EolinaRange, EolinaRange),
}

///
/// This is a helper range bound used instead of an [`isize`] to allow for `-0` relative from the
/// back indexing.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EolinaIndex {
    ///
    /// An absolute range bound `x` where `x` is a [`usize`].
    ///
    Start(usize),

    ///
    /// A range bound relative from the back `-x` where `x` is a [`usize`].
    ///
    End(usize),
}

impl EolinaIndex {
    ///
    /// Converts this [`EolinaIndex`] into an [`isize`], where a negative value denotes an index
    /// relative from the back.
    ///
    pub fn as_isize(&self) -> isize {
        match self {
            EolinaIndex::Start(idx) => *idx as isize,
            EolinaIndex::End(idx) => -(*idx as isize),
        }
    }

    ///
    /// Converts this [`EolinaIndex`] into an [`usize`], if it is within the bounds `0..len`.
    ///
    pub fn as_usize(&self, len: usize) -> Result<usize, IndexError> {
        match self {
            EolinaIndex::Start(idx) => {
                if *idx <= len {
                    Ok(*idx)
                } else {
                    Err(IndexError::OutOfTargetRange(*self, self.as_isize(), len))
                }
            }
            EolinaIndex::End(idx) => {
                if *idx <= len {
                    Ok(len - *idx)
                } else {
                    Err(IndexError::OutOfTargetRange(
                        *self,
                        len as isize + self.as_isize(),
                        len,
                    ))
                }
            }
        }
    }

    ///
    /// Creates a new [`EolinaIndex`] from the given components.
    ///
    pub fn from_components(from_back: bool, value: usize) -> Self {
        if from_back {
            Self::End(value)
        } else {
            Self::Start(value)
        }
    }
}

impl From<usize> for EolinaIndex {
    ///
    /// Creates a new [`EolinaIndex`] from the given [`usize`].
    ///
    fn from(int: usize) -> Self {
        Self::Start(int)
    }
}

impl From<isize> for EolinaIndex {
    ///
    /// Creates a new [`EolinaIndex`] from the given [`isize`].
    ///
    fn from(int: isize) -> Self {
        if int.is_negative() {
            Self::End(int.abs() as usize)
        } else {
            Self::Start(int as usize)
        }
    }
}

impl From<EolinaIndex> for isize {
    ///
    /// Creates a new [`isize`] from the given [`EolinaIndex`], where
    /// a negative value denotes an index relative from the back.
    ///
    fn from(bound: EolinaIndex) -> isize {
        bound.as_isize()
    }
}

impl Display for EolinaIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EolinaIndex::Start(idx) => Display::fmt(idx, f)?,
            EolinaIndex::End(idx) => {
                f.write_char('-')?;
                Display::fmt(idx, f)?;
            }
        }

        Ok(())
    }
}

///
/// A convinience range type for error handling and simpler [`Display`] impl. Is displayed as an
/// eolina range `|s.e|` where `s` is the inclusive relative start and `e` exclusive relative end.
/// Negative values denote relative from the end.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct EolinaRange {
    ///
    /// The optional inclusive start bound.
    ///
    pub start: Option<EolinaIndex>,

    ///
    /// The optional exclusive end bound.
    ///
    pub end: Option<EolinaIndex>,
}

impl EolinaRange {
    ///
    /// Creates a new [`EolinaIndex`] from the given bounds.
    ///
    pub fn components(start: Option<(bool, usize)>, end: Option<(bool, usize)>) -> Self {
        Self {
            start: start.map(|(rel, num)| EolinaIndex::from_components(rel, num)),
            end: end.map(|(rel, num)| EolinaIndex::from_components(rel, num)),
        }
    }

    ///
    /// Creates a new [`Range<usize>`] from this [`EolinaIndex`] if it's within
    /// a target range given the `len`, otherwise [`None`].
    ///
    pub fn as_range(&self, len: usize) -> Result<Range<usize>, RangeError> {
        // map to absolute
        let idx_map = |idx| match idx {
            EolinaIndex::Start(idx) => idx as isize,
            EolinaIndex::End(idx) => len as isize - idx as isize,
        };

        // check if valid
        let abs_check_map = |abs| {
            if (0..=len as isize).contains(&abs) {
                Ok(abs as usize)
            } else {
                Err(abs)
            }
        };

        // map to absolute unsigned
        let abs_lower = self.start.map(idx_map).map(abs_check_map).unwrap_or(Ok(0));
        let abs_upper = self.end.map(idx_map).map(abs_check_map).unwrap_or(Ok(len));

        // handle error cases
        let valid = (0..len).into();
        let (lower, upper) = match (abs_lower, abs_upper) {
            (Ok(ok), Err(err)) => {
                return Err(RangeError::OutOfTargetRange(
                    *self,
                    (ok as isize..err).into(),
                    valid,
                ));
            }
            (Err(err), Ok(ok)) => {
                return Err(RangeError::OutOfTargetRange(
                    *self,
                    (err..ok as isize).into(),
                    valid,
                ));
            }
            (Err(err_l), Err(err_u)) => {
                return Err(RangeError::OutOfTargetRange(
                    *self,
                    (err_l..err_u).into(),
                    valid,
                ));
            }
            (Ok(ok_l), Ok(ok_u)) => {
                if ok_l > ok_u {
                    return Err(RangeError::IncompatibleBounds(
                        (ok_l as isize..ok_u as isize).into(),
                        valid,
                    ));
                } else {
                    (ok_l, ok_u)
                }
            }
        };

        Ok(lower..upper)
    }
}

impl From<RangeFrom<usize>> for EolinaRange {
    ///
    /// Creates a new [`EolinaRange`] from the given [`RangeFrom`].
    ///
    fn from(range_from: RangeFrom<usize>) -> Self {
        Self {
            start: Some(range_from.start.into()),
            end: None,
        }
    }
}

impl From<RangeTo<usize>> for EolinaRange {
    ///
    /// Creates a new [`EolinaRange`] from the given [`RangeTo`].
    ///
    fn from(range_to: RangeTo<usize>) -> Self {
        Self {
            start: None,
            end: Some(range_to.end.into()),
        }
    }
}

impl From<Range<usize>> for EolinaRange {
    ///
    /// Creates a new [`EolinaRange`] from the given [`Range`].
    ///
    fn from(range: Range<usize>) -> Self {
        Self {
            start: Some(range.start.into()),
            end: Some(range.end.into()),
        }
    }
}

impl From<RangeFrom<isize>> for EolinaRange {
    ///
    /// Creates a new [`EolinaRange`] from the given [`RangeFrom`].
    ///
    fn from(range_from: RangeFrom<isize>) -> Self {
        Self {
            start: Some(range_from.start.into()),
            end: None,
        }
    }
}

impl From<RangeTo<isize>> for EolinaRange {
    ///
    /// Creates a new [`EolinaRange`] from the given [`RangeTo`].
    ///
    fn from(range_to: RangeTo<isize>) -> Self {
        Self {
            start: None,
            end: Some(range_to.end.into()),
        }
    }
}

impl From<Range<isize>> for EolinaRange {
    ///
    /// Creates a new [`EolinaRange`] from the given [`Range`].
    ///
    fn from(range: Range<isize>) -> Self {
        Self {
            start: Some(range.start.into()),
            end: Some(range.end.into()),
        }
    }
}

impl From<RangeFull> for EolinaRange {
    ///
    /// Creates a new [`EolinaRange`] from the given [`RangeFull`].
    ///
    fn from(_range_full: RangeFull) -> Self {
        Self {
            start: None,
            end: None,
        }
    }
}

impl Display for EolinaRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_char('|')?;
        if let Some(ref bound) = self.start {
            Display::fmt(bound, f)?;
        }
        f.write_char('.')?;
        if let Some(ref bound) = self.end {
            Display::fmt(bound, f)?;
        }
        f.write_char('|')?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn as_usize_valid() {
        let len = 4;
        let abs_index: EolinaIndex = 4isize.into();
        let rel_index: EolinaIndex = (-4isize).into();

        assert_eq!(abs_index.as_usize(len).unwrap(), 4);
        assert_eq!(rel_index.as_usize(len).unwrap(), 0);
    }

    #[test]
    fn as_usize_out_of_range() {
        let len = 4;
        let abs_index: EolinaIndex = 5isize.into();
        let rel_index: EolinaIndex = (-5isize).into();

        assert_eq!(
            abs_index.as_usize(len).unwrap_err(),
            IndexError::OutOfTargetRange(abs_index, 5, 4)
        );
        assert_eq!(
            rel_index.as_usize(len).unwrap_err(),
            IndexError::OutOfTargetRange(rel_index, -1, 4)
        );
    }

    #[test]
    fn as_range_valid() {
        let len = 4;
        let abs_abs_range: EolinaRange = (0..4isize).into();
        let rel_abs_range: EolinaRange = (-1..4isize).into();
        let abs_rel_range: EolinaRange = (0..-1isize).into();
        let rel_rel_range: EolinaRange = (-2..-1isize).into();

        assert_eq!(abs_abs_range.as_range(len).unwrap(), 0..4);
        assert_eq!(rel_abs_range.as_range(len).unwrap(), 3..4);
        assert_eq!(abs_rel_range.as_range(len).unwrap(), 0..3);
        assert_eq!(rel_rel_range.as_range(len).unwrap(), 2..3);
    }

    #[test]
    fn as_range_out_of_target() {
        let len = 4;
        let abs_abs_range: EolinaRange = (0..5isize).into();
        let rel_abs_range: EolinaRange = (-5..4isize).into();
        let abs_rel_range: EolinaRange = (0..-5isize).into();
        let rel_rel_range: EolinaRange = (-5..-5isize).into();

        assert_eq!(
            abs_abs_range.as_range(len).unwrap_err(),
            RangeError::OutOfTargetRange(abs_abs_range, (0..5isize).into(), (0..4isize).into())
        );
        assert_eq!(
            rel_abs_range.as_range(len).unwrap_err(),
            RangeError::OutOfTargetRange(rel_abs_range, (-1..4isize).into(), (0..4isize).into())
        );
        assert_eq!(
            abs_rel_range.as_range(len).unwrap_err(),
            RangeError::OutOfTargetRange(abs_rel_range, (0..-1isize).into(), (0..4isize).into())
        );
        assert_eq!(
            rel_rel_range.as_range(len).unwrap_err(),
            RangeError::OutOfTargetRange(rel_rel_range, (-1..-1isize).into(), (0..4isize).into())
        );
    }

    #[test]
    fn as_range_incompatible() {
        let len = 4;
        let abs_abs_range: EolinaRange = (4..0isize).into();
        let rel_abs_range: EolinaRange = (-1..0isize).into();
        let abs_rel_range: EolinaRange = (4..-4isize).into();
        let rel_rel_range: EolinaRange = (-1..-4isize).into();

        assert_eq!(
            abs_abs_range.as_range(len).unwrap_err(),
            RangeError::IncompatibleBounds((4..0isize).into(), (0..4isize).into())
        );
        assert_eq!(
            rel_abs_range.as_range(len).unwrap_err(),
            RangeError::IncompatibleBounds((3..0isize).into(), (0..4isize).into())
        );
        assert_eq!(
            abs_rel_range.as_range(len).unwrap_err(),
            RangeError::IncompatibleBounds((4..0isize).into(), (0..4isize).into())
        );
        assert_eq!(
            rel_rel_range.as_range(len).unwrap_err(),
            RangeError::IncompatibleBounds((3..0isize).into(), (0..4isize).into())
        );
    }
}
