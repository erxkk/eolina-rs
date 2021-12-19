use std::{
    fmt::{self, Debug, Display, Formatter, Write},
    ops::{Range, RangeFrom, RangeFull, RangeTo},
};

#[derive(thiserror::Error, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum Error {
    ///
    /// The given range bounds are incompatible after relaltive
    /// to absolute conversion. Contains the given relative range,
    /// and the computed absolute range.
    ///
    #[error("incompatible bounds: rel {0} => abs {1} (start > end)")]
    IncompatibleBounds(EolinaRange, EolinaRange),

    ///
    /// The given range bounds are out of the valid target range.
    /// Contains the given relative range, the absolute computed
    /// range and the valid range.
    ///
    #[error("out of range: rel {0} => abs {1} vs valid {2}")]
    OutOfTargetRange(EolinaRange, EolinaRange, EolinaRange),
}

///
/// This is a helper range bound used instead of an [`isize`] to allow for
/// `-0` relative from the back indexing.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EolinaRangeBound {
    ///
    /// An absolute range bound `x` where `x` is a [`usize`].
    ///
    Start(usize),

    ///
    /// A range bound relative from the back `-x` where `x` is a [`usize`].
    ///
    End(usize),
}

impl EolinaRangeBound {
    ///
    /// Converts this [`EolinaRangeBound`] into an [`isize`], where a negative
    /// value denotes an index relative from the back.
    ///
    pub fn as_isize(&self) -> isize {
        match self {
            EolinaRangeBound::Start(idx) => *idx as isize,
            EolinaRangeBound::End(idx) => -(*idx as isize),
        }
    }

    ///
    /// Creates a new [`EolinaRangeBound`] from the given components.
    ///
    pub fn from_components(from_back: bool, value: usize) -> Self {
        if from_back {
            Self::End(value)
        } else {
            Self::Start(value)
        }
    }
}

impl From<usize> for EolinaRangeBound {
    ///
    /// Creates a new [`EolinaRangeBound`] from the given [`usize`].
    ///
    fn from(int: usize) -> Self {
        Self::Start(int)
    }
}

impl From<isize> for EolinaRangeBound {
    ///
    /// Creates a new [`EolinaRangeBound`] from the given [`isize`].
    ///
    fn from(int: isize) -> Self {
        if int.is_negative() {
            Self::End(int.abs() as usize)
        } else {
            Self::Start(int as usize)
        }
    }
}

impl From<EolinaRangeBound> for isize {
    ///
    /// Creates a new [`isize`] from the given [`EolinaRangeBound`], where
    /// a negative value denotes an index relative from the back.
    ///
    fn from(bound: EolinaRangeBound) -> isize {
        bound.as_isize()
    }
}

impl Display for EolinaRangeBound {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EolinaRangeBound::Start(idx) => Display::fmt(idx, f)?,
            EolinaRangeBound::End(idx) => {
                f.write_char('-')?;
                Display::fmt(idx, f)?;
            }
        }

        Ok(())
    }
}

///
/// A convinience range type for error handling and simpler [`Display`] impl.
/// Is displayed as an eolina range `|s.e|` where `s` is the inclusive relative
/// start and `e` exclusive relative end. Negative values denote relative from
/// the end.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct EolinaRange {
    ///
    /// The optional start bound.
    ///
    pub start: Option<EolinaRangeBound>,

    ///
    /// The optional end bound.
    ///
    pub end: Option<EolinaRangeBound>,
}

impl EolinaRange {
    ///
    /// Creates a new [`EolinaRangeBound`] from the given bounds.
    ///
    pub fn components(start: Option<(bool, usize)>, end: Option<(bool, usize)>) -> Self {
        Self {
            start: start.map(|(rel, num)| EolinaRangeBound::from_components(rel, num)),
            end: end.map(|(rel, num)| EolinaRangeBound::from_components(rel, num)),
        }
    }

    ///
    /// Creates a new [`Range<usize>`] from this [`EolinaRangeBound`] if it's within
    /// a target range given the `len`, otherwise [`None`].
    ///
    pub fn as_range(&self, len: usize) -> Result<Range<usize>, Error> {
        // map to absolute
        let idx_map = |idx| match idx {
            EolinaRangeBound::Start(idx) => idx as isize,
            EolinaRangeBound::End(idx) => len as isize - idx as isize,
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
                return Err(Error::OutOfTargetRange(
                    *self,
                    (ok as isize..err).into(),
                    valid,
                ));
            }
            (Err(err), Ok(ok)) => {
                return Err(Error::OutOfTargetRange(
                    *self,
                    (err..ok as isize).into(),
                    valid,
                ));
            }
            (Err(err_l), Err(err_u)) => {
                return Err(Error::OutOfTargetRange(*self, (err_l..err_u).into(), valid));
            }
            (Ok(ok_l), Ok(ok_u)) => {
                if ok_l > ok_u {
                    return Err(Error::IncompatibleBounds(
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
    fn to_range_valid() {
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
    fn to_range_out_of_target() {
        let len = 4;
        let abs_abs_range: EolinaRange = (0..5isize).into();
        let rel_abs_range: EolinaRange = (-5..4isize).into();
        let abs_rel_range: EolinaRange = (0..-5isize).into();
        let rel_rel_range: EolinaRange = (-5..-5isize).into();

        assert_eq!(
            abs_abs_range.as_range(len).unwrap_err(),
            Error::OutOfTargetRange(abs_abs_range, (0..5isize).into(), (0..4isize).into())
        );
        assert_eq!(
            rel_abs_range.as_range(len).unwrap_err(),
            Error::OutOfTargetRange(rel_abs_range, (-1..4isize).into(), (0..4isize).into())
        );
        assert_eq!(
            abs_rel_range.as_range(len).unwrap_err(),
            Error::OutOfTargetRange(abs_rel_range, (0..-1isize).into(), (0..4isize).into())
        );
        assert_eq!(
            rel_rel_range.as_range(len).unwrap_err(),
            Error::OutOfTargetRange(rel_rel_range, (-1..-1isize).into(), (0..4isize).into())
        );
    }

    #[test]
    fn to_range_incompatible() {
        let len = 4;
        let abs_abs_range: EolinaRange = (4..0isize).into();
        let rel_abs_range: EolinaRange = (-1..0isize).into();
        let abs_rel_range: EolinaRange = (4..-4isize).into();
        let rel_rel_range: EolinaRange = (-1..-4isize).into();

        assert_eq!(
            abs_abs_range.as_range(len).unwrap_err(),
            Error::IncompatibleBounds((4..0isize).into(), (0..4isize).into())
        );
        assert_eq!(
            rel_abs_range.as_range(len).unwrap_err(),
            Error::IncompatibleBounds((3..0isize).into(), (0..4isize).into())
        );
        assert_eq!(
            abs_rel_range.as_range(len).unwrap_err(),
            Error::IncompatibleBounds((4..0isize).into(), (0..4isize).into())
        );
        assert_eq!(
            rel_rel_range.as_range(len).unwrap_err(),
            Error::IncompatibleBounds((3..0isize).into(), (0..4isize).into())
        );
    }
}
