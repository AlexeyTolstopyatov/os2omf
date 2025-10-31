use std::io::{self, Read};
use crate::types::PascalString;

///
/// This table contains a list of ASCII strings.
///
/// The first string is the module name given in the module definition file.
///
/// The other strings are the names of all exported functions listed in the module definition
/// file that were not given explicit @ordinal numbers or
/// that were explicitly specified in the file as resident names.
/// (Exported functions with explicit ordinal numbers in the module definition file
/// are listed in the nonresident names table.)
///
/// Each string is prefaced by a single byte indicating the number of
/// characters in the string and is followed by a word (2 bytes)
/// referencing an element in the entry table, beginning at 1. The word
/// that follows the module name is 0. (Offsets are from the beginning of
/// the record.)
///
#[derive(Debug, Clone)]
pub struct ResidentNameTable {
    pub entries: Vec<ResidentNameEntry>,
}

impl ResidentNameTable {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut entries = Vec::new();
        while let Some(entry) = ResidentNameEntry::read(r)? {
            entries.push(entry);
        }
        Ok(Self { entries })
    }
}

#[derive(Debug, Clone)]
pub struct ResidentNameEntry {
    pub name: PascalString,
    pub ordinal: u16,
}

impl ResidentNameEntry {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Option<Self>> {
        // TODO: make it general. "resident names" and "not resident names" are the same structures but have different locations.
        let len = {
            let mut len = 0;
            r.read_exact(std::slice::from_mut(&mut len))?;
            len
        };
        if len == 0 {
            return Ok(None);
        }
        let name = {
            let mut name = vec![0; len as usize];
            r.read_exact(&mut name)?;
            name
        };
        let index = {
            let mut buf = [0; 2];
            r.read_exact(&mut buf)?;
            u16::from_le_bytes(buf)
        };
        Ok(Some(Self {
            name: PascalString::new(len, name),
            ordinal: index
        }))
    }
}