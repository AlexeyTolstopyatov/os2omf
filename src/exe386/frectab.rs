use crate::exe386::fpagetab::FixupPageTable;
use std::io::{self, Error, ErrorKind, Read, Seek, SeekFrom};

#[derive(Debug, Clone)]
pub struct FixupRecord {
    pub source: u8,
    pub target_flags: u8,
    pub source_offset_or_count: u16,
    pub target_data: FixupTarget,
    pub additive_value: Option<u32>,
    pub source_offset_list: Option<Vec<u16>>,
}

#[derive(Debug, Clone)]
pub enum FixupTarget {
    Internal(FixupTargetInternal),
    ImportedOrdinal(FixupTargetImportedOrdinal),
    ImportedName(FixupTargetImportedName),
    FixupViaEntryTable(FixupTargetEntryTable),
}

#[derive(Debug, Clone)]
pub struct FixupTargetInternal {
    pub object_number: u16,
    pub target_offset: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct FixupTargetImportedOrdinal {
    pub module_ordinal: u16,
    pub import_ordinal: u32, // <-- might be 8/16/32-bit
}

#[derive(Debug, Clone)]
pub struct FixupTargetImportedName {
    pub module_ordinal: u16,
    pub procedure_name_offset: u32,
}

#[derive(Debug, Clone)]
pub struct FixupTargetEntryTable {
    pub entry_number: u16,
}

#[derive(Debug, Clone)]
pub struct FixupFlags {
    pub has_source_list: bool,
    pub has_additive: bool,
    pub is_32bit_target: bool,
    pub is_32bit_additive: bool,
    pub is_16bit_object_module: bool,
    pub is_8bit_ordinal: bool,
    pub target_type: u8,
    pub source_type: u8,
}

impl FixupFlags {
    pub fn from_bytes(source: u8, target_flags: u8) -> Self {
        FixupFlags {
            has_source_list: (source & 0x20) != 0,
            has_additive: (target_flags & 0x04) != 0,
            is_32bit_target: (target_flags & 0x10) != 0,
            is_32bit_additive: (target_flags & 0x20) != 0,
            is_16bit_object_module: (target_flags & 0x40) != 0,
            is_8bit_ordinal: (target_flags & 0x80) != 0,
            target_type: target_flags & 0x03,
            source_type: source & 0x0F,
        }
    }
}

pub struct FixupRecordsTable {
    pub records: Vec<FixupRecord>,
}

impl FixupRecordsTable {
    pub fn read<R: Read + Seek>(
        reader: &mut R,
        fixup_page_table: &FixupPageTable,
        fixup_record_table_offset: u64,
    ) -> io::Result<Self> {
        let mut records = Vec::new();

        for (logical_page, &page_offset) in fixup_page_table.page_offsets.iter().enumerate() {
            let record_offset = fixup_record_table_offset + page_offset as u64;
            reader.seek(SeekFrom::Start(record_offset))?;

            // I can read records till next page offset!
            // For elsewhere it throws unexpected problems
            
            let next_offset = fixup_page_table.page_offsets.get(logical_page + 1)
                .copied()
                .unwrap_or(fixup_page_table.end_of_fixup_records);

            while reader.stream_position()? < fixup_record_table_offset + next_offset as u64 {
                if let Some(record) = Self::read_single_fixup_record(reader)? {
                    records.push(record);
                } else {
                    break;
                }
            }
        }

        Ok(Self { 
            records 
        })
    }

    fn read_single_fixup_record<R: Read>(reader: &mut R) -> io::Result<Option<FixupRecord>> {
        let mut source_buf = [0_u8];

        reader.read_exact(&mut source_buf)?;

        let source = source_buf[0];

        let mut target_flags_buf = [0_u8];
        reader.read_exact(&mut target_flags_buf)?;
        let target_flags = target_flags_buf[0];

        let flags = FixupFlags::from_bytes(source, target_flags);

        let source_offset_or_count = if flags.has_source_list {
            let mut count_buf = [0_u8];
            reader.read_exact(&mut count_buf)?;
            count_buf[0] as u16
        } else {
            let mut offset_buf = [0_u8; 2];
            reader.read_exact(&mut offset_buf)?;
            u16::from_le_bytes(offset_buf)
        };

        let target_data = Self::read_target_data(reader, &flags)?;
        let additive_value = if flags.has_additive {
            Some(if flags.is_32bit_additive {
                let mut additive_buf = [0_u8; 4];
                reader.read_exact(&mut additive_buf)?;
                u32::from_le_bytes(additive_buf)
            } else {
                let mut additive_buf = [0_u8; 2];
                reader.read_exact(&mut additive_buf)?;
                u16::from_le_bytes(additive_buf) as u32
            })
        } else {
            None
        };

        let source_offset_list = if flags.has_source_list {
            let count = source_offset_or_count as usize;
            let mut list = Vec::with_capacity(count);
            for _ in 0..count {
                let mut offset_buf = [0_u8; 2];
                reader.read_exact(&mut offset_buf)?;
                list.push(u16::from_le_bytes(offset_buf));
            }
            Some(list)
        } else {
            None
        };

        Ok(Some(FixupRecord {
            source,
            target_flags,
            source_offset_or_count,
            target_data,
            additive_value,
            source_offset_list,
        }))
    }

    fn read_target_data<R: Read>(reader: &mut R, flags: &FixupFlags) -> io::Result<FixupTarget> {
        match flags.target_type {
            0x00 => Self::read_internal_target(reader, flags),
            0x01 => Self::read_imported_ordinal_target(reader, flags),
            0x02 => Self::read_imported_name_target(reader, flags),
            0x03 => Self::read_entry_table_target(reader, flags),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Unknown target type: 0x{:02x}", flags.target_type),
            )),
        }
    }

    fn read_internal_target<R: Read>(reader: &mut R, flags: &FixupFlags) -> io::Result<FixupTarget> {
        let object_number = match flags.is_16bit_object_module {
            true => {
                let mut obj_buf = [0_u8; 2];
                reader.read_exact(&mut obj_buf)?;
                u16::from_le_bytes(obj_buf)
            }
            false => {
                let mut obj_buf = [0_u8];
                reader.read_exact(&mut obj_buf)?;
                obj_buf[0] as u16
            }
        };

        let target_offset = if flags.source_type != 0x02 {
            Some(match flags.is_32bit_target {
                true => {
                    let mut offset_buf = [0_u8; 4];
                    reader.read_exact(&mut offset_buf)?;
                    u32::from_le_bytes(offset_buf)
                }
                false => {
                    let mut offset_buf = [0_u8; 2];
                    reader.read_exact(&mut offset_buf)?;
                    u16::from_le_bytes(offset_buf) as u32
                }
            })
        } else {
            None
        };

        Ok(FixupTarget::Internal(FixupTargetInternal {
            object_number,
            target_offset,
        }))
    }

    fn read_imported_ordinal_target<R: Read>(
        reader: &mut R,
        flags: &FixupFlags,
    ) -> io::Result<FixupTarget> {
        let module_ordinal = match flags.is_16bit_object_module {
            true => {
                let mut mod_buf = [0_u8; 2];
            reader.read_exact(&mut mod_buf)?;
                u16::from_le_bytes(mod_buf)
            }
            false => {
                let mut mod_buf = [0_u8];
                reader.read_exact(&mut mod_buf)?;
                mod_buf[0] as u16
            }
        };

        let import_ordinal = if flags.is_8bit_ordinal {
            let mut ordinal_buf = [0_u8];
            reader.read_exact(&mut ordinal_buf)?;
            ordinal_buf[0] as u32
        } else if flags.is_32bit_target {
            let mut ordinal_buf = [0_u8; 4];
            reader.read_exact(&mut ordinal_buf)?;
            u32::from_le_bytes(ordinal_buf)
        } else {
            let mut ordinal_buf = [0_u8; 2];
            reader.read_exact(&mut ordinal_buf)?;
            u16::from_le_bytes(ordinal_buf) as u32
        };

        Ok(FixupTarget::ImportedOrdinal(FixupTargetImportedOrdinal {
            module_ordinal,
            import_ordinal,
        }))
    }

    fn read_imported_name_target<R: Read>(
        reader: &mut R,
        flags: &FixupFlags,
    ) -> io::Result<FixupTarget> {
        let module_ordinal = match flags.is_16bit_object_module {
            true => {
                let mut mod_buf = [0_u8; 2];
            reader.read_exact(&mut mod_buf)?;
                u16::from_le_bytes(mod_buf)
            }
            false => {
                let mut mod_buf = [0_u8];
                reader.read_exact(&mut mod_buf)?;
                mod_buf[0] as u16
            }
        };

        let procedure_name_offset = match flags.is_32bit_target {
            true => {
                let mut offset_buf = [0_u8; 4];
                reader.read_exact(&mut offset_buf)?;
                u32::from_le_bytes(offset_buf)
            }
            false => {
                let mut offset_buf = [0_u8; 2];
                reader.read_exact(&mut offset_buf)?;
                u16::from_le_bytes(offset_buf) as u32
            }
        };

        Ok(FixupTarget::ImportedName(FixupTargetImportedName {
            module_ordinal,
            procedure_name_offset,
        }))
    }

    fn read_entry_table_target<R: Read>(
        reader: &mut R,
        flags: &FixupFlags,
    ) -> io::Result<FixupTarget> {
        let entry_number = match flags.is_16bit_object_module {
            true => {
                let mut entry_buf = [0_u8; 2];
                reader.read_exact(&mut entry_buf)?;
                u16::from_le_bytes(entry_buf)
            }
            false => {
                let mut entry_buf = [0_u8];
                reader.read_exact(&mut entry_buf)?;
                entry_buf[0] as u16
            }
        };


        Ok(FixupTarget::FixupViaEntryTable(FixupTargetEntryTable {
            entry_number,
        }))
    }
}