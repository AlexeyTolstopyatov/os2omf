//! This module represents API of nonresident names table
use crate::types::PascalString;
use std::io;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct NonResidentNameEntry {
    pub name: PascalString,
    pub ordinal: u16,
}

impl NonResidentNameEntry {
    pub fn read<TRead: Read>(r: &mut TRead) -> io::Result<Option<Self>> {
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
            r.read_exact(name.as_mut_slice())?;
            name
        };
        let index = {
            let mut buf = [0; 2];
            r.read_exact(&mut buf)?;
            u16::from_le_bytes(buf)
        };
        Ok(Some(Self {
            name: PascalString::new(len, name),
            ordinal: index,
        }))
    }
}
