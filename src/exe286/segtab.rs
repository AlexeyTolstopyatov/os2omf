use crate::exe286::segrelocs::{RelocationTable, RelocationType};
use crate::types::PascalString;
use std::io::{self, Read, Seek, SeekFrom};
///
/// This table contains one 8-byte record for every code and data segment
/// in the program or library module.
///
/// Each segment has an #ordinal number
/// associated with it. For example, the first segment has an ordinal
/// number of 1.
/// These segment numbers are used to reference the segments
/// in other sections of the New Executable file.
/// (Offsets are from the beginning of the record.)
///
#[derive(Debug, Clone)]
pub struct NeSegment {
    pub header: NeSegmentHeader,
    pub shift_count: u16,
    pub data: Option<Vec<u8>>,
    pub relocs: RelocationTable,
}

impl NeSegment {
    pub fn read<T: Read + Seek>(reader: &mut T, alignment: u16) -> io::Result<Self> {
        let alignment = if alignment == 0 { 9 } else { alignment };
        let header = NeSegmentHeader::read(reader)?;

        let relocs = if !header.relocations_stripped() {
            Self::read_relocs(reader, alignment.into(), &header)?
        } else {
            RelocationTable { rel_entries: vec![] }
        };

        Ok(Self {
            header,
            shift_count: alignment,
            data: None,
            relocs,
        })
    }

    fn read_relocs<T: Read + Seek>(
        reader: &mut T,
        alignment: u64,
        header: &NeSegmentHeader
    ) -> io::Result<RelocationTable> {
        let position = match (header.sector_base as u64).checked_mul(1 << alignment) {
            Some(base_shifted) => base_shifted.checked_add(header.sector_length as u64),
            None => None,
        };

        let position = match position {
            Some(pos) => pos,
            None => return Ok(RelocationTable { rel_entries: vec![] }),
        };

        let current_pos = reader.stream_position()?;
        let file_length = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(current_pos))?;

        if position + 2 > file_length {
            return Ok(RelocationTable { rel_entries: vec![] });
        }

        reader.seek(SeekFrom::Start(position))?;
        RelocationTable::read(reader)
    }

    pub fn read_data<T: Read + Seek>(&mut self, reader: &mut T) -> io::Result<()> {
        if self.header.sector_base == 0 {
            return Ok(());
        }

        let data_offset = self.header.data_offset(self.shift_count.into());
        let data_length = self.header.data_length();

        reader.seek(SeekFrom::Start(data_offset))?;
        let mut data = vec![0; data_length as usize];
        reader.read_exact(&mut data)?;
        self.data = Some(data);

        Ok(())
    }
}

// Более идиоматичная реализация для DllImport
impl DllImport {
    pub fn new(dll_name: PascalString, name: PascalString, ordinal: u16, file_pointer: u64) -> Self {
        Self {
            dll_name,
            name,
            ordinal,
            file_pointer,
        }
    }
}

/// ### Imports extraction from segmented module
/// Read [it](https://alexeytolstopyatov.github.io/notes/2025/09/23/ne-imptab.html) please
/// if you really need to know how to define dynamic imports
pub struct NeSegmentDllImportsTable {
    pub seg_number: i32,
    pub imp_list: Vec<DllImport>,
}
impl NeSegmentDllImportsTable {
    pub fn read<T: Read + Seek>(
        reader: &mut T,
        relocs: &RelocationTable,
        imp_tab: u32,
        mod_tab: u32,
        seg_number: i32,
    ) -> io::Result<Self> {
        let mut imp_list = Vec::new();

        for reloc in &relocs.rel_entries {
            match &reloc.rel_type {
                RelocationType::ImportName(import_name) => {
                    if let Some(import) = Self::read_import_name(
                        reader, import_name, imp_tab, mod_tab
                    )? {
                        imp_list.push(import);
                    }
                }
                RelocationType::ImportOrdinal(import_ord) => {
                    if let Some(import) = Self::read_import_ordinal(
                        reader, import_ord, imp_tab, mod_tab
                    )? {
                        imp_list.push(import);
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            seg_number,
            imp_list,
        })
    }

    fn read_import_name<T: Read + Seek>(
        reader: &mut T,
        import_name: &crate::exe286::segrelocs::ImportName,
        imp_tab: u32,
        mod_tab: u32,
    ) -> io::Result<Option<DllImport>> {
        let mod_offset = Self::read_module_offset(reader, mod_tab, import_name.imp_mod)?;
        let mod_offset = match mod_offset {
            Some(offset) => offset,
            None => return Ok(None),
        };

        let dll_name = Self::read_module_name(reader, imp_tab, mod_offset)?;
        let proc_name = Self::read_procedure_name(reader, imp_tab, import_name.imp_offset)?;

        Ok(Some(DllImport::new(
            dll_name,
            proc_name,
            0,
            (imp_tab + import_name.imp_offset as u32) as u64,
        )))
    }

    fn read_import_ordinal<T: Read + Seek>(
        reader: &mut T,
        import_ord: &crate::exe286::segrelocs::ImportOrdinal,
        imp_tab: u32,
        mod_tab: u32,
    ) -> io::Result<Option<DllImport>> {
        let mod_offset = Self::read_module_offset(reader, mod_tab, import_ord.imp_mod_index)?;
        let mod_offset = match mod_offset {
            Some(offset) => offset,
            None => return Ok(None),
        };

        let dll_name = Self::read_module_name(reader, imp_tab, mod_offset)?;

        Ok(Some(DllImport::new(
            dll_name,
            PascalString::empty(),
            import_ord.imp_ordinal,
            reader.stream_position()?,
        )))
    }

    fn read_module_offset<T: Read + Seek>(
        reader: &mut T,
        mod_tab: u32,
        imp_mod: u16,
    ) -> io::Result<Option<u16>> {
        let mod_offset_ptr = mod_tab + 2 * (imp_mod - 1) as u32;
        reader.seek(SeekFrom::Start(mod_offset_ptr as u64))?;

        let mut mod_offset_buf = [0; 2];
        reader.read_exact(&mut mod_offset_buf)?;
        let mod_offset = u16::from_le_bytes(mod_offset_buf);

        Ok(if mod_offset == 0 { None } else { Some(mod_offset) })
    }

    fn read_module_name<T: Read + Seek>(
        reader: &mut T,
        imp_tab: u32,
        mod_offset: u16,
    ) -> io::Result<PascalString> {
        let mod_ptr = imp_tab + mod_offset as u32;
        reader.seek(SeekFrom::Start(mod_ptr as u64))?;

        let mut mod_len = 0;
        reader.read_exact(std::slice::from_mut(&mut mod_len))?;

        let mut name = vec![0; mod_len as usize];
        reader.read_exact(&mut name)?;

        Ok(PascalString::new(mod_len, name))
    }

    fn read_procedure_name<T: Read + Seek>(
        reader: &mut T,
        imp_tab: u32,
        imp_offset: u16,
    ) -> io::Result<PascalString> {
        let proc_ptr = imp_tab + imp_offset as u32;
        reader.seek(SeekFrom::Start(proc_ptr as u64))?;

        let mut proc_len = 0;
        reader.read_exact(std::slice::from_mut(&mut proc_len))?;

        if proc_len == 0 {
            return Ok(PascalString::empty());
        }

        let mut name = vec![0; proc_len as usize];
        reader.read_exact(&mut name)?;

        Ok(PascalString::new(proc_len, name))
    }
}
///
/// NE Segment header is a record in Segments table
/// Like a PE32/+ files, NE executable images has a table of something which
/// contains raw code or data.
/// ```
///                                            |
/// +--------+--------+-------+----------+ <---+ e_lfanew + e_segtab
/// | offset | length | flags | minalloc |
/// | 0xABCD | 0x0100 | 0x... | ...      |
/// | 0xBOOC | 0x0020 | 0x007 | ...      |
/// | ...    | ...    | ...   | ...      |
///      |                 |
///      |                 |
///      |                 +-----> Based on flags and SEG_HASMASK (0x0007) byte
///  Segments with offset = 0      defines the rules for each segment in table.
///  are .BSS prototypes           flags & HASMASK = 1 -> .CODE16 segment
///  because there's no iterated                     0 -> .DATA16 segment
///  or compressed segments       (flags & PRELOAD) + (flags & HASMASK)
///                                                 0 -> .DATA16  (read-write)
///                                                 1 -> .RDATA16 (read-only)
/// ```
/// Every segment has a rights to contain own relocations table,
/// because this way to imagine the segments table is most simple.
///
#[derive(Debug, Clone, Copy)]
pub struct NeSegmentHeader {
    pub sector_base: u16,
    pub sector_length: u16,
    pub flags: u16,
    pub min_alloc: u16,
}
///
/// Segments in NE segmented executable are unnamed. Every segment
/// has flags what describes it. Types following next don't try
/// compare with sections in Portable Executable format but.
///
/// Portable Executables works in private process-memory and
/// memory model is flat. Segmented memory model is significant
/// thing.
///
pub enum NeSegmentRights {
    /// Rights of 16-bit .code segment
    ///  - READABLE
    ///  - EXECUTABLE
    CODE = 0,
    /// Rights of 16-bit .data segment
    ///  - READABLE
    ///  - WRITABLE
    DATA = 1,
    /// Rights of 16-bit .rdata segment
    ///  - READABLE
    RDATA= 2,
    ///
    /// My custom new-type of segment without
    /// embedded data. In original documents there's no
    /// any .bss named segments and all segments are unnamed by nature,
    /// Rights of 16-bit ~.bss~ data segment defines by the flags
    ///
    BSS = 3
}
const SEG_HASMASK: u16 = 0x0007;
///
/// Segment marked as moveable can be moved into another segment
/// after application loads into Windows memory.
///
const SEG_MOVABLE: u16 = 0x0010;
///
/// Data segments having this flag are read-only
/// All segments marked as SEG_PRELOAD are loads in memory before
/// Windows loader prepares to run application.
///
const SEG_PRELOAD: u16 = 0x0040;
///
/// If byte-mask of segment OR SEG_RELOCS gives true -
/// next following data of segment is will be huge table of
/// segment relocations. Per-segment relocations is very important tables
/// which describe all inner-relocations and statically linked functions
/// and dynamically linked libraries/functions/calls in run-time used in module.
///
/// If application requires FPU -> Windows emulates it and contains special
/// marks in per-segment relocations named like "OS-Fixups".
///
const SEG_RELOCS:  u16 = 0x0100;
///
/// If segment marked as discardable - it can be unloaded
/// after application runs.
///
const SEG_DISCARD: u16 = 0xF000;

impl NeSegmentHeader {
    ///
    /// Reads only one segment and fills unsafe unaligned structure
    /// of one segment header
    ///
    pub fn read<TRead: Read>(r: &mut TRead) -> io::Result<Self> {
        let mut buf = [0; 0x8];
        r.read_exact(&mut buf)?;
        let get_u16 = |pos| u16::from_le_bytes(buf[pos..pos + 2]
            .try_into()
            .unwrap());

        Ok(Self {
            sector_base: get_u16(0),
            sector_length: get_u16(2),
            flags: get_u16(4),
            min_alloc: get_u16(6),
        })
    }
    ///
    /// Compares all byte-mask with current flags of
    /// segment rights.
    ///
    pub fn get_segment_rights(&self) -> NeSegmentRights {
        if self.sector_base == 0 {
            return NeSegmentRights::BSS;
        }

        match (self.flags & SEG_HASMASK) != 0 {
            true => NeSegmentRights::CODE,
            false => {
                if (self.flags & SEG_PRELOAD) != 0 {
                    NeSegmentRights::RDATA
                } else {
                    NeSegmentRights::DATA
                }
            }
        }
    }
    ///
    /// Remember the NE Header `e_align` field??
    /// This is a main reason of usage this field. Per-segment relocations
    /// depend hard on sector shifting
    ///
    pub fn data_offset(&self, alignment: u64) -> u64 {
        (self.sector_base as u64) << alignment
    }

    pub fn data_length(&self) -> u64 {
        if self.sector_length == 0 {
            0x10000
        } else {
            self.sector_length as u64
        }
    }

    pub fn min_alloc(&self) -> u64 {
        if self.min_alloc == 0 {
            0x10000
        } else {
            self.min_alloc as u64
        }
    }
    pub fn relocations_stripped(&self) -> bool {
        (self.flags & SEG_RELOCS) == 0
    }
}

/// > This scheme is custom!
///
/// It's not include in official documentation.
pub struct DllImport {
    /// ### Module's Name
    /// Module's name after linker distorts and becomes `PASCALUPPERCASE`
    /// Historically, Microsoft and IBM use `PascalCase` naming for procedures
    /// and for functions written in C/++ modules. This rule figures out everywhere
    /// but Microsoft LINK.EXE corrupts it.
    ///
    /// Module Names are not DLL file names! If you read my articles about or
    /// Microsoft official manual for "Segmented Executables" module names implicitly
    /// casts to a `@0` record in `ResidentNames` table. That's main reason why `@0`
    /// ordinal is a reserved value.
    ///
    /// ```
    /// KERNEL.EXE may contain "KERNEL" pascal string in resident names table
    ///            by @0 ordinal. And this is a module name exactly.
    ///
    /// ```
    ///
    /// You can rename KERNEL.EXE to KERNEL.DLL or something else, but system's loader
    /// looks up at the @0 ordinal **if module defined** _and_ **required to be loaded**
    pub dll_name: PascalString,
    ///
    /// ### Procedure's Name
    ///
    /// If you want to know more about it: pls read [it](https://alexeytolstopyatov.github.io/notes/2025/09/23/ne-imptab.html)
    /// I've described all problems and base of it there.
    pub name: PascalString,
    /// ### Procedure's Ordinal
    /// Uses instead name if entry point is unnamed or
    /// specially hidden by linker in special project file ".def"
    ///
    /// Exports in another modules declares the Name of entry point
    /// and positioning index in the EntryTable. This index calls by others "ordinal".
    ///
    pub ordinal: u16,
    pub file_pointer: u64,
}