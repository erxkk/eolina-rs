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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn char_into_lower() {
        assert_eq!('a'.into_lower(), 'a');
        assert_eq!('B'.into_lower(), 'b');
        assert_eq!('6'.into_lower(), '6');
    }

    #[test]
    fn char_into_upper() {
        assert_eq!('a'.into_upper(), 'A');
        assert_eq!('B'.into_upper(), 'B');
        assert_eq!('6'.into_upper(), '6');
    }

    #[test]
    fn char_into_swap() {
        assert_eq!('a'.into_swap(), 'A');
        assert_eq!('B'.into_swap(), 'b');
        assert_eq!('6'.into_swap(), '6');
    }

    #[test]
    fn char_is_lower() {
        assert!(!'A'.to_owned().is_lower());
        assert!('b'.to_owned().is_lower());
        assert!(!'6'.to_owned().is_lower());
    }

    #[test]
    fn char_is_upper() {
        assert!('A'.to_owned().is_upper());
        assert!(!'b'.to_owned().is_upper());
        assert!(!'6'.to_owned().is_upper());
    }

    #[test]
    fn char_is_vowel() {
        assert!('a'.to_owned().is_vowel());
        assert!(!'B'.to_owned().is_vowel());
        assert!(!'!'.to_owned().is_vowel());
    }

    #[test]
    fn char_is_conso() {
        assert!(!'a'.to_owned().is_conso());
        assert!('B'.to_owned().is_conso());
        assert!(!'!'.to_owned().is_conso());
    }

    #[test]
    fn string_into_lower() {
        assert_eq!("aA".to_owned().into_lower(), "aa");
        assert_eq!("Bb".to_owned().into_lower(), "bb");
        assert_eq!("6!".to_owned().into_lower(), "6!");
    }

    #[test]
    fn string_into_upper() {
        assert_eq!("aA".to_owned().into_upper(), "AA");
        assert_eq!("Bb".to_owned().into_upper(), "BB");
        assert_eq!("6!".to_owned().into_upper(), "6!");
    }

    #[test]
    fn string_into_swap() {
        assert_eq!("aA".to_owned().into_swap(), "Aa");
        assert_eq!("Bb".to_owned().into_swap(), "bB");
        assert_eq!("6!".to_owned().into_swap(), "6!");
    }

    #[test]
    fn string_is_lower() {
        assert!("abc".to_owned().is_lower());
        assert!(!"Abc".to_owned().is_lower());
        assert!(!"6!".to_owned().is_lower());
    }

    #[test]
    fn string_is_upper() {
        assert!(!"aBc".to_owned().is_upper());
        assert!("ABC".to_owned().is_upper());
        assert!(!"6!".to_owned().is_upper());
    }

    #[test]
    fn string_is_vowel() {
        assert!("aEi".to_owned().is_vowel());
        assert!(!"Abe".to_owned().is_vowel());
        assert!(!"aB!".to_owned().is_vowel());
    }

    #[test]
    fn string_is_conso() {
        assert!(!"aBi".to_owned().is_conso());
        assert!("bcD".to_owned().is_conso());
        assert!(!"Bc!".to_owned().is_conso());
    }
}
