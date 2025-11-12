// Unsafe undeclared types may contain here

mod readable;
use std::fmt::Debug;

///
/// ### Pascal String
/// Type of ASCII string mostly used in Pascal.
/// Pascal string always has first byte with all string length.
///
/// That's main difference between it and terminated C-Strings
/// ```pas
/// str = "pascal string" -> bytes[0] - 13
///                          bytes[1] - 'p'
///                          bytes[2] - 'a'
///                          bytes[3] - 's'
///                          ...
///                          bytes[13] - 'g'
/// ```
///
#[derive(Debug, Clone)]
pub(crate) struct PascalString {
    length: u8,
    string: Vec<u8>,
}
impl PascalString {
    pub fn empty() -> Self {
        PascalString {
            length: 0,
            string: Vec::new(),
        }
    }
    pub fn new(len: u8, bytes: Vec<u8>) -> Self {
        PascalString {
            length: len,
            string: bytes,
        }
    }
    pub fn to_string(&self) -> String {
        std::str::from_utf8(&self.string).expect("\0").to_string()
    }
    pub fn to_bytes(&self) -> &[u8] {
        self.string.as_slice()
    }
}
//
// impl Debug for PascalString {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.to_string())
//     }
// }