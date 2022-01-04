mod map;
pub use map::Map;

mod predicate;
pub use predicate::Predicate;

use super::Inline;
use std::fmt::{self, Display, Formatter, Write};

///
/// A transformation of a value.
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Transform<'p> {
    ///
    /// The filter token `f[x]` where `x` is a [`Predicate`] token.
    ///
    Filter(Predicate<'p>),

    ///
    /// A map token `m[x]` where `x` is a [`Map`] token.
    ///
    Map(Map),

    ///
    /// A [`Predicate`] token `p[x]`.
    ///
    Predicate(Predicate<'p>),

    // TODO: join only allows string literal and block
    ///
    /// A join token `.[x]` where `x` is a literal, `.` is allowed.
    ///
    Join(Option<Inline<'p>>),

    ///
    /// A split token `/[x]` where `x` is a literal, `/` is allowed.
    ///
    Split(Option<Inline<'p>>),
}

impl<'p> Display for Transform<'p> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let print_arg = |f: &mut Formatter<'_>, arg: &dyn Display| write!(f, "[{}]", arg);

        match self {
            Self::Filter(arg) => {
                f.write_char('f')?;
                print_arg(f, arg)?;
            }
            Self::Map(arg) => {
                f.write_char('m')?;
                print_arg(f, arg)?;
            }
            Self::Predicate(arg) => {
                arg.fmt(f)?;
            }
            Self::Join(arg) => {
                f.write_char('.')?;
                if let Some(arg) = arg {
                    print_arg(f, arg)?;
                }
            }
            Self::Split(arg) => {
                f.write_char('/')?;
                if let Some(arg) = arg {
                    print_arg(f, arg)?;
                }
            }
        }

        Ok(())
    }
}
