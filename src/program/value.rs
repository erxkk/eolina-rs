use std::fmt::{self, Debug, Display, Formatter};

///
/// The kind of a [`Value`].
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Kind {
    ///
    /// A [`String`].
    ///
    String,

    ///
    /// A vec of [`String`]s.
    ///
    StringVec,

    ///
    /// A [`bool`].
    ///
    Bool,
}

impl Display for Kind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::String => f.write_str("String"),
            Self::StringVec => f.write_str("StringVec"),
            Self::Bool => f.write_str("Bool"),
        }
    }
}

///
/// A value accepted or returned by a function.
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
    /// Returns the [`Kind`] of this [`Value`].
    ///
    pub fn kind(&self) -> Kind {
        match self {
            Self::String(_) => Kind::String,
            Self::StringVec(_) => Kind::StringVec,
            Self::Bool(_) => Kind::Bool,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            // use debug version to show explicit delimiters
            Self::String(inner) => Debug::fmt(inner, f),
            Self::StringVec(inner) => Debug::fmt(inner, f),
            Self::Bool(inner) => Display::fmt(inner, f),
        }
    }
}
