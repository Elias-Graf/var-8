use std::fmt::Debug;

pub const VARIATION_SELECTOR: &str = "\u{fe0f}";
pub const ZERO_WIDTH_JOINER: &str = "\u{200d}";

#[derive(PartialEq, Eq)]
pub struct UTF8Char<'a> {
    bytes: &'a [u8],
}

impl<'a> UTF8Char<'a> {
    pub fn as_str(&self) -> &'a str {
        // SAFETY: The [`UTF8Char`] should only be constructed with valid bytes.
        unsafe { std::str::from_utf8_unchecked(self.bytes) }
    }

    /// Compare the `&str` representation against a given string slice.
    pub fn is(&self, right: &str) -> bool {
        self.as_str() == right
    }
}

impl<'a> Debug for UTF8Char<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", self.as_str())
    }
}

impl<'a> From<&'a str> for UTF8Char<'a> {
    fn from(val: &'a str) -> Self {
        UTF8Char {
            bytes: val.as_bytes(),
        }
    }
}

#[derive(Debug)]
pub struct UTF8Chars<'a> {
    bytes: &'a [u8],
}

impl<'a> Iterator for UTF8Chars<'a> {
    type Item = UTF8Char<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.is_empty() {
            return None;
        }

        let (cp_bytes, mut remaining_bytes) = next_code_point(self.bytes)?;
        let mut utf8_char_len = cp_bytes.len();

        // Variation selector
        if let Some((variant_bytes, variant_remaining)) = variation_selector(remaining_bytes) {
            utf8_char_len += variant_bytes.len();
            remaining_bytes = variant_remaining;
        }

        // Zero with joiner
        // A utf-8 char can consist out of one or more joined together code points.
        while let Some((joined_bytes, join_remaining)) = zero_width_joiner(remaining_bytes) {
            utf8_char_len += joined_bytes.len();
            remaining_bytes = join_remaining;
        }

        let utf8_char = UTF8Char {
            bytes: &self.bytes[..utf8_char_len],
        };

        self.bytes = remaining_bytes;
        Some(utf8_char)
    }
}

fn variation_selector(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    let (cp_bytes, remainder) = next_code_point(bytes)?;
    let cp = bytes_as_str(cp_bytes);

    if cp == VARIATION_SELECTOR {
        return Some((cp_bytes, remainder));
    }

    None
}

fn zero_width_joiner(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    let (joiner_bytes, remainder) = next_code_point(bytes)?;
    let joiner = bytes_as_str(joiner_bytes);

    if joiner != ZERO_WIDTH_JOINER {
        return None;
    }

    if let Some((join_with_bytes, remainder)) = next_code_point(remainder) {
        return Some((
            &bytes[..joiner_bytes.len() + join_with_bytes.len()],
            remainder,
        ));
    }

    // If this is reached, no code point followed the joiner. For now it's just
    // "gracefully" ignored.
    // TODO: figure out what the standard for error handling is.
    Some((joiner_bytes, remainder))
}

fn next_code_point(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    if bytes.is_empty() {
        return None;
    }

    Some(bytes.split_at(code_point_len(bytes)))
}

pub trait ToUTF8Chars<'a> {
    fn utf8_chars(&'a self) -> UTF8Chars<'a>;
}

impl<'a> ToUTF8Chars<'a> for str {
    fn utf8_chars(&'a self) -> UTF8Chars<'a> {
        UTF8Chars {
            bytes: self.as_bytes(),
        }
    }
}

/// Get the length of a code point.
///
/// # Panics
///
/// If the following assumptions are incorrect:
///
/// - The function is called with the start of the code point at index 0.
/// - The function is called with valid utf-8 bytes.
pub fn code_point_len(bytes: &[u8]) -> usize {
    let first_byte = &bytes[0];

    if first_byte & 0b1000_0000 == 0 {
        return 1;
    }

    if first_byte & 0b1110_0000 == 0b1100_0000 {
        return 2;
    }

    if first_byte & 0b1111_0000 == 0b1110_0000 {
        return 3;
    }

    if first_byte & 0b1111_1000 == 0b1111_0000 {
        return 4;
    }

    panic!("invalid first byte '{:08b}'", first_byte);
}

/// Converts the given byte array to an utf-8 string.
///
/// This conversion assumes that valid utf-8 bytes are passed.
fn bytes_as_str(bytes: &[u8]) -> &str {
    // SAFETY: This function should only be called with valid utf-8 bytes.
    unsafe { std::str::from_utf8_unchecked(bytes) }
}

#[cfg(test)]
mod tests {
    use crate::{code_point_len, ToUTF8Chars};

    #[test]
    fn single_byte_code_point_len() {
        assert_eq!(code_point_len("A".as_bytes()), 1);
    }

    #[test]
    fn double_byte_code_point_len() {
        assert_eq!(code_point_len("±".as_bytes()), 2);
    }

    #[test]
    fn triple_byte_code_point_len() {
        assert_eq!(code_point_len("⚽".as_bytes()), 3);
    }

    #[test]
    fn quadruple_byte_code_point_len() {
        assert_eq!(code_point_len("🫥".as_bytes()), 4);
    }

    #[test]
    fn utf8_chars() {
        let mut chars = "A±⚽🫥".utf8_chars();

        assert_eq!(chars.next(), Some("A".into()));
        assert_eq!(chars.next(), Some("±".into()));
        assert_eq!(chars.next(), Some("⚽".into()));
        assert_eq!(chars.next(), Some("🫥".into()));
        assert_eq!(chars.next(), None);
    }

    #[test]
    fn utf8_char_single_zero_width_joiner() {
        let mut chars = "👨‍🦰".utf8_chars();

        assert_eq!(chars.next(), Some("👨‍🦰".into()));
        assert_eq!(chars.next(), None);
    }

    #[test]
    fn utf8_char_multiple_zero_width_joiners() {
        let mut chars = "👨‍👩‍👦".utf8_chars();

        assert_eq!(chars.next(), Some("👨‍👩‍👦".into()));
        assert_eq!(chars.next(), None);
    }

    #[test]
    fn utf_8_char_variation_selector_zero_width_joiner() {
        let mut chars = "🏳️‍🌈".utf8_chars();

        assert_eq!(chars.next(), Some("🏳️‍🌈".into()));
        assert_eq!(chars.next(), None);
    }
}
