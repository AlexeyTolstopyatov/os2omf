//! This module represents generics of per-segment relocations
//! New Executable format defines four types of per-segment relocation
//!  - Internal
//!  - Run-time Import by name
//!  - Run-time Import by index (ordinal)
//!  - FPU fixup
//! 
//! Internal fixup is a far-pointer (pointer to another segment) which
//! applies to struct/procedure or any entry.
//! 
//! Records which contains information about imports are stores pointers
//! to procedure call and procedure symbol. 
//! 
//! FPU fixups are instructions what Windows
//! wants to "fix-up" while application runs
use std::io;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct InternalFixup {
    pub int_seg: u8,
    /// Is moveable?
    pub int_mov: bool,
    pub int_offset: u16,
}
#[derive(Clone, Debug)]
pub struct ImportOrdinal {
    pub imp_mod_index: u16,
    pub imp_ordinal: u16,
}
#[derive(Clone, Debug)]
pub struct ImportName {
    pub imp_mod_index: u16,
    pub imp_offset: u16,
}
#[derive(Clone, Debug)]
pub struct FPUFixup {
    /// See FPUFixupType
    pub osf_type: FPUFixupType,
    /// unused space. Usually 0x0000
    pub osf_padd: u16,
}
///
/// This is a type of instruction what Windows
/// wants to "fix-up" while application runs
///
/// Type what marked as "j" will be second in
/// command/opcode sequence
///
#[derive(Clone, Debug)]
#[repr(u16)]
pub enum FPUFixupType {
    FiArqqFjArqq = 0x0001,
    FiSrqqFjSrqq = 0x0002,
    FiCrqqFjCrqq = 0x0003,
    FiErqq = 0x0004,
    FiDrqq = 0x0005,
    FiWrqq = 0x0006,
}
impl FPUFixupType {
     pub fn get_from(u: u16) -> FPUFixupType {
        match u {
            0x0001 => FPUFixupType::FiArqqFjArqq,
            0x0002 => FPUFixupType::FiSrqqFjSrqq,
            0x0003 => FPUFixupType::FiCrqqFjCrqq,
            0x0004 => FPUFixupType::FiErqq,
            0x0005 => FPUFixupType::FiDrqq,
            _ => FPUFixupType::FiDrqq,
        }
    }
}
#[derive(Debug, Clone)]
pub enum RelocationType {
    Internal(InternalFixup),
    ImportName(ImportName),
    ImportOrdinal(ImportOrdinal),
    OSFixup(FPUFixup),
}
///
/// Every relocation record in table of relocations
/// is fixed-size entry.
///
/// Derivatives of RelocationType are having 32-bit size
/// And it helps to define size of all table avoiding shit.
///
#[derive(Debug, Clone)]
pub struct RelocationEntry {
    pub rel_rtp: u8,   // Address Type
    pub rel_atp: u8,   // Relocation type
    pub rel_add: bool, // Is Additive?
    pub rel_seg_ptr: u16,
    pub rel_type: RelocationType,
}
///
/// Relocation table is a sequence of defined
/// relocation records.
///
#[derive(Debug, Clone)]
pub struct RelocationTable {
    pub rel_entries: Vec<RelocationEntry>,
}

impl RelocationTable {
    pub fn read<TRead: Read>(r: &mut TRead) -> io::Result<Self> {
        let mut count_buf = [0; 2];
        r.read_exact(&mut count_buf)?;
        let count: u16 = bytemuck::cast(count_buf);

        let mut entries = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let mut entry_buf = [0; 8];
            r.read_exact(&mut entry_buf)?;

            let address_type = entry_buf[0];
            let reloc_flags = entry_buf[1];
            let reloc_type = reloc_flags & 0x03; // Lower 2 bits
            let is_additive = (reloc_flags & 0x04) != 0; // Bit 2
            let segment_offset = u16::from_le_bytes([entry_buf[2], entry_buf[3]]);

            let target = match reloc_type {
                // Internal reference
                0x00 => {
                    let segment = entry_buf[4];
                    let is_movable = segment == 0xFF;
                    let offset_or_ordinal = u16::from_le_bytes([entry_buf[6], entry_buf[7]]);

                    let internal_fix: InternalFixup = InternalFixup {
                        int_seg: segment,
                        int_mov: is_movable,
                        int_offset: offset_or_ordinal,
                    };

                    RelocationType::Internal(internal_fix)
                }
                // Imported by ordinal
                0x01 => {
                    let module_index = u16::from_le_bytes([entry_buf[4], entry_buf[5]]);
                    let ordinal = u16::from_le_bytes([entry_buf[6], entry_buf[7]]);

                    let import_by_ordinal: ImportOrdinal = ImportOrdinal {
                        imp_mod_index: module_index,
                        imp_ordinal: ordinal,
                    };

                    RelocationType::ImportOrdinal(import_by_ordinal)
                }
                // Imported by name
                0x02 => {
                    let module_index = u16::from_le_bytes([entry_buf[4], entry_buf[5]]);
                    let name_offset = u16::from_le_bytes([entry_buf[6], entry_buf[7]]);

                    let import_by_name = ImportName {
                        imp_mod_index: module_index,
                        imp_offset: name_offset,
                    };

                    RelocationType::ImportName(import_by_name)
                }
                // FPU emulation
                0x03 => {
                    let fixup = u16::from_le_bytes([entry_buf[4], entry_buf[5]]);
                    let null = u16::from_le_bytes([entry_buf[6], entry_buf[7]]);

                    let osf = FPUFixup {
                        osf_padd: null,
                        osf_type: FPUFixupType::get_from(fixup),
                    };
                    RelocationType::OSFixup(osf)
                }
                // unknown record?!
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("invalid relocation type: 0x{:02X}", reloc_type),
                    ));
                }
            };

            entries.push(RelocationEntry {
                rel_atp: address_type,
                rel_rtp: reloc_type,
                rel_add: is_additive,
                rel_seg_ptr: segment_offset,
                rel_type: target,
            });
        }

        Ok(Self {
            rel_entries: entries,
        })
    }
}
