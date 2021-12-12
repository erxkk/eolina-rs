use super::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

///
/// Represents a value accepted or returned by a function.
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value {
    ///
    /// A [`String`].
    ///
    String(String),

    ///
    /// A vec of [`String`]s.
    ///
    StringVec(Vec<String>),

    ///
    /// A [`bool`].
    ///
    Bool(bool),
}

impl Value {
    ///
    /// Returns the kind of this value.
    ///
    pub fn kind(&self) -> ValueKind {
        match self {
            Self::String(_) => ValueKind::String,
            Self::StringVec(_) => ValueKind::StringVec,
            Self::Bool(_) => ValueKind::Bool,
        }
    }

    ///
    /// Unwraps a string or returns a mismatch error.
    ///
    pub fn unwrap_string(self) -> Result<String, Error> {
        match self {
            Self::String(inner) => Ok(inner),
            x => Err(Error::arg_mismatch(&[ValueKind::String], x.kind())),
        }
    }

    ///
    /// Unwraps a string vec or returns a mismatch error.
    ///
    pub fn unwrap_string_vec(self) -> Result<Vec<String>, Error> {
        match self {
            Self::StringVec(inner) => Ok(inner),
            x => Err(Error::arg_mismatch(&[ValueKind::String], x.kind())),
        }
    }

    ///
    /// Unwraps a bool or returns a mismatch error.
    ///
    pub fn unwrap_bool(self) -> Result<bool, Error> {
        match self {
            Self::Bool(inner) => Ok(inner),
            x => Err(Error::arg_mismatch(&[ValueKind::String], x.kind())),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::String(inner) => Display::fmt(inner, f),
            Self::StringVec(inner) => Debug::fmt(inner, f),
            Self::Bool(inner) => Display::fmt(inner, f),
        }
    }
}

///
/// Represents the type of a value.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ValueKind {
    ///
    /// A string.
    ///
    String,
    ///
    /// A array fo strings.
    ///
    StringVec,
    ///
    /// A boolean.
    ///
    Bool,
}

impl Display for ValueKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::String => f.write_str("String"),
            Self::StringVec => f.write_str("StringVec"),
            Self::Bool => f.write_str("Bool"),
        }
    }
}
