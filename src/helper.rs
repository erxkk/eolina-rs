///
/// An extension trait for checking checking ascii chars for specific properties.
///
pub trait AsciiCheckExt {
    ///
    /// Converts [`self`] to it's uppercase represenation.
    ///
    fn to_upper(&self) -> Self;

    ///
    /// Converts [`self`] to it's lowercase represenation.
    ///
    fn to_lower(&self) -> Self;

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

impl AsciiCheckExt for u8 {
    fn to_upper(&self) -> Self {
        if self.is_lower() {
            *self
        } else {
            *self - (b'a' - b'A')
        }
    }

    fn to_lower(&self) -> Self {
        if self.is_upper() {
            *self + (b'a' - b'A')
        } else {
            *self
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

impl AsciiCheckExt for char {
    fn to_upper(&self) -> Self {
        self.to_ascii_uppercase()
    }

    fn to_lower(&self) -> Self {
        self.to_ascii_lowercase()
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

impl AsciiCheckExt for String {
    fn to_upper(&self) -> Self {
        self.to_uppercase()
    }

    fn to_lower(&self) -> Self {
        self.to_lowercase()
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
