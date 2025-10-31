use std::io::{self, Read, Seek, SeekFrom};
use crate::exe286::segrelocs::RelocationTable;

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
    pub relocs: RelocationTable
}

impl NeSegment {
    /// Reads the record in segments table
    /// without raw segment data.
    pub fn read<TRead: Read + Seek>(r: &mut TRead, shift_count: u16) -> io::Result<Self> {
        let header = NeSegmentHeader::read(r)?;
        let mut relocs = RelocationTable { rel_entries: vec![] };

        if !header.relocations_stripped() {
            relocs = Self::read_relocs(r, shift_count as u64, &header)
        }

        Ok(Self {
            header,
            shift_count,
            data: None,
            relocs
        })
    }
    fn read_relocs<TRead: Read + Seek>(r: &mut TRead, a: u64, h: &NeSegmentHeader) -> RelocationTable {
        // header already exists in memory I suppose...
        let position = h.data_offset(a) + h.data_length();

        if (position + 2) as usize == r.bytes().count() {
            RelocationTable { rel_entries: vec![] }
        }


    }
    /// Reads the segment data uses header information.
    pub fn read_data<TSeek: Read + Seek>(&mut self, r: &mut TSeek) -> io::Result<()> {
        if self.header.data_offset_shifted == 0 {
            return Ok(());
        }
        let data_offset = self.header.data_offset(self.shift_count as u64);
        let data_length = self.header.data_length();
        r.seek(SeekFrom::Start(data_offset))?;
        let mut data = vec![0; data_length as usize];
        r.read_exact(&mut data)?;
        self.data = Some(data);
        
        Ok(())
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
    pub data_offset_shifted: u16,
    pub data_length: u16,
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
            data_offset_shifted: get_u16(0),
            data_length: get_u16(2),
            flags: get_u16(4),
            min_alloc: get_u16(6),
        })
    }
    ///
    /// Compares all byte-mask with current flags of
    /// segment rights.
    ///
    pub fn get_segment_rights(&self) -> NeSegmentRights {
        if self.data_offset_shifted == 0_u16 {
            return NeSegmentRights::BSS;
        }

        match (self.flags & SEG_HASMASK) == 0 {
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
        (self.data_offset_shifted as u64) << alignment
    }

    pub fn data_length(&self) -> u64 {
        if self.data_length == 0 {
            0x10000
        } else {
            self.data_length as u64
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
    pub dll_name: String,
    ///
    /// ### Procedure's Name
    ///
    /// If you want to know more about it: pls read [it](https://alexeytolstopyatov.github.io/notes/2025/09/23/ne-imptab.html)
    /// I've described all problems and base of it there.
    pub name: String,
    /// ### Procedure's Ordinal
    /// Uses instead name if entry point is unnamed or
    /// specially hidden by linker in special project file ".def"
    ///
    /// Exports in another modules declares the Name of entry point
    /// and positioning index in the EntryTable. This index calls by others "ordinal".
    ///
    /// ```def
    /// NAME        = 'hello' # I can not stand NULL terminator to the strings here.
    ///                       # But for every string must have NULL terminator
    /// DESCRIPTION = 'This project linked under Windows 11 and VSCode'
    ///
    /// STACKSIZE   = 1024
    /// HEAPSIZE    = 4096
    ///
    /// EXPORTS
    /// HelloWatcom @60  # <-- Will be placed to Non-Resident Names
    ///                  #     as HELLOWATCOM by 60 position in EntryTable.
    /// ```
    pub ordinal: u16,
    pub file_pointer: u64,
}

/// ### Imports extraction from segmented module
/// Read [it](https://alexeytolstopyatov.github.io/notes/2025/09/23/ne-imptab.html) please
/// if you really need to know how to define dynamic imports
pub struct NeSegmentDllImportsTable {
    pub imp_list: Vec<DllImport>,
}