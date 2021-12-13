use std::{
    fmt::{self, Display, Formatter, Write},
    ops::{Range, RangeFrom, RangeFull, RangeTo},
};

///
/// An extension trait for checking checking ascii chars for specific
/// properties.
///
pub trait AsciiExt {
    ///
    /// Converts [`self`] to it's uppercase represenation.
    ///
    fn into_upper(self) -> Self;

    ///
    /// Converts [`self`] to it's lowercase represenation.
    ///
    fn into_lower(self) -> Self;

    ///
    /// Converts [`self`] to it's lowercase represenation if it was
    /// uppercase and vice versa.
    ///
    fn into_swap(self) -> Self;

    ///
    /// Returns whether [`self`] is in it's uppercase represenation.
    ///
    fn is_upper(&self) -> bool;

    ///
    /// Returns whether [`self`] is in it's lowercase represenation.
    ///
    fn is_lower(&self) -> bool;

    ///
    /// Returns whether [`self`] is or contains only vowels.
    ///
    fn is_vowel(&self) -> bool;

    ///
    /// Returns whether [`self`] is or contains only consonants.
    ///
    fn is_conso(&self) -> bool;
}

impl AsciiExt for u8 {
    fn into_upper(self) -> Self {
        if self.is_lower() {
            self
        } else {
            self - (b'a' - b'A')
        }
    }

    fn into_lower(self) -> Self {
        if self.is_upper() {
            self + (b'a' - b'A')
        } else {
            self
        }
    }

    fn into_swap(self) -> Self {
        if self.is_upper() {
            self.into_lower()
        } else if self.is_lower() {
            self.into_upper()
        } else {
            self
        }
    }

    fn is_lower(&self) -> bool {
        matches!(*self, b'a'..=b'z')
    }

    fn is_upper(&self) -> bool {
        matches!(*self, b'A'..=b'Z')
    }

    fn is_vowel(&self) -> bool {
        matches!(
            *self,
            b'a' | b'e' | b'i' | b'o' | b'u' | b'A' | b'E' | b'I' | b'O' | b'U'
        )
    }

    fn is_conso(&self) -> bool {
        matches!(
            *self,
            b'b'..=b'd'
            | b'f'..=b'h'
            | b'j'..=b'n'
            | b'p'..=b't'
            | b'v'..=b'z'
            | b'B'..=b'D'
            | b'F'..=b'H'
            | b'J'..=b'N'
            | b'P'..=b'T'
            | b'V'..=b'Z'
        )
    }
}

impl AsciiExt for char {
    fn into_upper(self) -> Self {
        self.to_ascii_uppercase()
    }

    fn into_lower(self) -> Self {
        self.to_ascii_lowercase()
    }

    fn into_swap(self) -> Self {
        if self.is_upper() {
            self.into_lower()
        } else if self.is_lower() {
            self.into_upper()
        } else {
            self
        }
    }

    fn is_lower(&self) -> bool {
        self.is_ascii_lowercase()
    }

    fn is_upper(&self) -> bool {
        self.is_ascii_uppercase()
    }

    fn is_vowel(&self) -> bool {
        matches!(
            *self,
            'a' | 'e' | 'i' | 'o' | 'u' | 'A' | 'E' | 'I' | 'O' | 'U'
        )
    }

    fn is_conso(&self) -> bool {
        matches!(
            *self,
            'b'..='d'
            | 'f'..='h'
            | 'j'..='n'
            | 'p'..='t'
            | 'v'..='z'
            | 'B'..='D'
            | 'F'..='H'
            | 'J'..='N'
            | 'P'..='T'
            | 'V'..='Z'
        )
    }
}

impl AsciiExt for String {
    fn into_upper(self) -> Self {
        self.to_uppercase()
    }

    fn into_lower(self) -> Self {
        self.to_lowercase()
    }

    fn into_swap(self) -> Self {
        self.chars().map(|ch| ch.into_swap()).collect()
    }

    fn is_upper(&self) -> bool {
        self.chars().all(|ch| ch.is_upper())
    }

    fn is_lower(&self) -> bool {
        self.chars().all(|ch| ch.is_lower())
    }

    fn is_vowel(&self) -> bool {
        self.chars().all(|ch| ch.is_vowel())
    }

    fn is_conso(&self) -> bool {
        self.chars().all(|ch| ch.is_conso())
    }
}

///
/// This is a helper range bound used instead of an [`isize`] to allow for
/// -0` relative from the back indexing.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EolinaRangeBound {
    ///
    /// An absolute range bound `x` where `x` is a [`usize`]/
    ///
    Start(usize),

    ///
    /// A range bound relative from the back `-x` where `x` is a [`usize`]/
    ///
    End(usize),
}

impl EolinaRangeBound {
    ///
    /// Converts this [`EolinaRangeBound`] into an [`isize`], where a negative
    /// value denotes a relative index.
    ///
    pub fn to_isize(&self) -> isize {
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

impl Into<isize> for EolinaRangeBound {
    fn into(self) -> isize {
        self.to_isize()
    }
}

impl Display for EolinaRangeBound {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EolinaRangeBound::Start(idx) => idx.fmt(f)?,
            EolinaRangeBound::End(idx) => {
                f.write_char('-')?;
                idx.fmt(f)?;
            }
        }

        Ok(())
    }
}

///
/// A convinience range type for error handling and simpler [`Display`] impl.
/// Is displayed as an eolina range `|s.e|` where `s` is the inclusive relateive start and `e` exclusive relative the end.
/// Negative values denote relative from the end.
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
    pub fn new(start: Option<EolinaRangeBound>, end: Option<EolinaRangeBound>) -> Self {
        Self { start, end }
    }

    ///
    /// Creates a new [`EolinaRangeBound`] from the given bounds.
    ///
    pub fn from_isize(start: Option<isize>, end: Option<isize>) -> Self {
        Self {
            start: start.map(|num| num.into()),
            end: end.map(|num| num.into()),
        }
    }

    ///
    /// Creates a new [`EolinaRangeBound`] from the given bounds.
    ///
    pub fn from_components(start: Option<(bool, usize)>, end: Option<(bool, usize)>) -> Self {
        Self {
            start: start.map(|(rel, num)| EolinaRangeBound::from_components(rel, num)),
            end: end.map(|(rel, num)| EolinaRangeBound::from_components(rel, num)),
        }
    }

    ///
    /// Applies a the given mapping functions to both bounds if they are [`Some`].
    ///
    pub fn map(
        self,
        f_start: impl Fn(EolinaRangeBound) -> EolinaRangeBound,
        f_end: impl Fn(EolinaRangeBound) -> EolinaRangeBound,
    ) -> Self {
        Self {
            start: self.start.map(f_start),
            end: self.end.map(f_end),
        }
    }
}

impl From<RangeFrom<isize>> for EolinaRange {
    ///
    /// Creates a new [`OptionRange`] from the given [`RangeFrom`].
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
    /// Creates a new [`OptionRange`] from the given [`RangeTo`].
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
    /// Creates a new [`OptionRange`] from the given [`Range`].
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
    /// Creates a new [`OptionRange`] from the given [`RangeFull`].
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
            bound.fmt(f)?;
        }
        f.write_char('.')?;
        if let Some(ref bound) = self.end {
            bound.fmt(f)?;
        }
        f.write_char('|')?;
        Ok(())
    }
}
