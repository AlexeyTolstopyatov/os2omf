//! This module represents structure and methods of FixupPages table
use crate::exe386::frectab::FixupRecord;
use crate::exe386::header::LinearExecutableHeader;
use std::io;
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug, Clone)]
pub struct FixupPageTable {
    pub page_offsets: Vec<u32>,
    pub end_of_fixup_records: u32,
}
#[derive(Debug, Clone)]
pub struct FixupRecordTable {
    pub records: Vec<FixupRecord>,
}

impl FixupPageTable {
    pub fn read<R: Read + Seek>(
        reader: &mut R,
        fpagetab: u64,
        header: &LinearExecutableHeader,
    ) -> io::Result<Self> {
        reader.seek(SeekFrom::Start(fpagetab))?;
        if header.e32_fpagetab == 0 {
            return Ok(Self {
                page_offsets: Vec::new(),
                end_of_fixup_records: 0,
            });
        }

        // records = fpages + 1 (needed end marker too)
        let entry_count = header.e32_mpages as usize + 1;

        let mut page_offsets = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            let mut buf = [0_u8; 4];
            reader.read_exact(&mut buf)?;
            page_offsets.push(u32::from_le_bytes(buf));
        }

        let end_of_fixup_records = page_offsets.pop().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "No end marker in fixup page table",
            )
        })?;

        Ok(Self {
            page_offsets,
            end_of_fixup_records,
        })
    }
}
