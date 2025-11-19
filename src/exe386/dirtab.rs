use crate::exe386::header::LinearExecutableHeader;
use bytemuck::{Pod, Zeroable};
use std::io;
use std::io::{Read, Seek, SeekFrom};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct ModuleDirectiveRecord {
    pub directive_number: u16,
    pub data_length: u16,
    pub data_offset: u32,
}

#[derive(Debug, Clone)]
pub struct ModuleDirective {
    pub directive_type: DirectiveType,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum DirectiveType {
    VerifyRecord,
    LanguageInfo,
    CoprocessorRequired,
    ThreadStateInit,
    Unknown(u16),
}
impl From<u16> for DirectiveType {
    fn from(value: u16) -> Self {
        match value {
            0x8001 => DirectiveType::VerifyRecord,
            0x0002 => DirectiveType::LanguageInfo,
            0x0003 => DirectiveType::CoprocessorRequired,
            0x0004 => DirectiveType::ThreadStateInit,
            n => DirectiveType::Unknown(n),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VerifyRecord {
    pub module_dependencies: Vec<ModuleDependency>,
}

#[derive(Debug, Clone)]
pub struct ModuleDependency {
    pub module_ordinal: u16,
    pub version: u16,
    pub module_object_count: u16,
    pub object_verifications: Vec<ObjectVerification>,
}

#[derive(Debug, Clone)]
pub struct ObjectVerification {
    pub object_number: u16,
    pub base_address: u32,
    pub virtual_size: u32,
}

pub struct ModuleDirectivesTable {
    directives: Vec<ModuleDirective>,
}

impl ModuleDirectivesTable {
    pub fn empty() -> Self {
        Self {
            directives: Vec::new(),
        }
    }
    pub fn read<R: Read + Seek>(
        reader: &mut R,
        header: &LinearExecutableHeader,
        e_lfanew: u64,
    ) -> io::Result<Self> {
        if header.e32_impmod == 0 || header.e32_impmodcnt == 0 {
            return Ok(Self {
                directives: Vec::new(),
            });
        }

        reader.seek(SeekFrom::Start(header.e32_impmod as u64 + e_lfanew))?;

        let mut directives = Vec::with_capacity(header.e32_impmodcnt as usize);
        for _ in 0..header.e32_impmodcnt {
            let mut entry_buf = [0u8; 8];
            reader.read_exact(&mut entry_buf)?;
            let entry: ModuleDirectiveRecord = bytemuck::pod_read_unaligned(&entry_buf);

            // Directive data
            let directive_type = DirectiveType::from(entry.directive_number);
            let mut data = vec![0u8; entry.data_length as usize];

            let data_offset = if entry.directive_number & 0x8000 != 0 {
                // Resident table - offset from header
                header.e32_magic as u64 + entry.data_offset as u64
            } else {
                // Non-resident table - offset from file start
                entry.data_offset as u64
            };

            let current_pos = reader.stream_position()?;
            reader.seek(SeekFrom::Start(data_offset))?;
            reader.read_exact(&mut data)?;
            reader.seek(SeekFrom::Start(current_pos))?;

            directives.push(ModuleDirective {
                directive_type,
                data,
            });
        }

        Ok(Self { directives })
    }

    pub fn read_verify_record(directive: &ModuleDirective) -> io::Result<VerifyRecord> {
        if !matches!(directive.directive_type, DirectiveType::VerifyRecord) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not a verify record directive",
            ));
        }

        let data = &directive.data;
        if data.len() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Verify record too short",
            ));
        }

        let entry_count = u16::from_le_bytes([data[0], data[1]]) as usize;
        let mut dependencies = Vec::with_capacity(entry_count);
        let mut offset = 2;

        for _ in 0..entry_count {
            if offset + 8 > data.len() {
                break;
            }

            let module_ordinal = u16::from_le_bytes([data[offset], data[offset + 1]]);
            let version = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
            let module_object_count = u16::from_le_bytes([data[offset + 4], data[offset + 5]]);
            offset += 6;

            let mut object_verifications = Vec::with_capacity(module_object_count as usize);
            for _ in 0..module_object_count {
                if offset + 8 > data.len() {
                    break;
                }

                let object_number = u16::from_le_bytes([data[offset], data[offset + 1]]);
                let base_address = u32::from_le_bytes([
                    data[offset + 2],
                    data[offset + 3],
                    data[offset + 4],
                    data[offset + 5],
                ]);
                let virtual_size = u32::from_le_bytes([
                    data[offset + 6],
                    data[offset + 7],
                    data[offset + 8],
                    data[offset + 9],
                ]);
                offset += 10;

                object_verifications.push(ObjectVerification {
                    object_number,
                    base_address,
                    virtual_size,
                });
            }

            dependencies.push(ModuleDependency {
                module_ordinal,
                version,
                module_object_count,
                object_verifications,
            });
        }

        Ok(VerifyRecord {
            module_dependencies: dependencies,
        })
    }
}
