use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Seek, SeekFrom};
use crate::exe286::enttab::{Entry, EntryTable};
use crate::exe286::header::NeHeader;
use crate::exe286::modtab::NeModuleReferencesTable;
use crate::exe286::nrestab::NonResidentNameTable;
use crate::exe286::resntab::{ResidentNameEntry, ResidentNameTable};
use crate::exe286::segrelocs::RelocationTable;
use crate::exe286::segtab::{NeSegment, NeSegmentDllImportsTable};
use crate::exe::MzHeader;

pub const NE_MAGIC: u16 = 0x454e;
pub const NE_CIGAM: u16 = 0x4e45;

// connect modules "files" here
pub mod header;
pub mod modtab; 
pub mod segtab;
pub mod segrelocs;
pub mod enttab;
pub mod nrestab;
pub mod resntab;
/// ### Segmented New Executable Layout
/// Every segmented OS/2-Windows executable is a book with specific data inside
/// This book traditionally has table of content
/// Main regions of this book is a segments like sections in PE32/+ or ELF32/64 files
/// 
/// ```
/// +--------+--------+---------+
/// | MZ |...|e_lfarlc|e_lfanew ------+
/// +--------+----|---+---------+     |
/// |             | always eq.  |     | **Absolute offset** what holds in e_lfanew
/// |             | 0x40 (64)   |     | is an raw file pointer to next structure
/// |[      ]<----+             |     | 
/// |                           |     | That's why it calls e_lfanew.
/// |                     +-----+     | "long file address (of) new executable"
/// |                     | NE  <-----+
/// +---+---+---+---+-----+-----+
/// |lnk|lnk|...|...| ... |     | 
/// +---+---+---+---+-----+-----+
/// | other fields  | winver[2] |
/// +---------------------------+
/// |        padding            |
/// +---------------------------+
/// | .CODE segment 1           | **Segments Table** of New Executable contains not just
/// | .CODE segment 2           | segments data of length and positions. For each segment
/// | .DATA segment 3 []        | in table if flags byte mask contains SEG_HASRELOC (0x0100)
/// |                   +-------+ exists following next array of relocations.
/// +-------------------+       |
/// |        padding            |
/// +---------------------------+
/// | 01 | 06 | 11 | 78 | 20 |  | **Module References Table**
/// +----+----+-------+----+----+
/// | 00 | 03 | "GDI" | 03 | MSG| **Importing modules** strings
/// +----+----+-------+----+----+ 
/// | 09 | FATALEXIT  |         | **Resident names** (private exports)
/// +---------------------------+    | or exports used by module in runtime
/// | 08 | ABOUT_RC | @1 | ...  | <--+
/// |----+----------+----+------+ 
/// | #1 | E_SHARED | E...      | <-- EntryPoints Table
/// | [... ... ...] | #2 E_UNUSED     Main table for all exports in module  
/// | [... ...] | ... +---------|     holds positions and to every entry
/// +-----------------+         |     in all registered segments in file.
/// |  Segments and paddings    | 
/// | raw data and x86 are here |  
/// +---------------------------+
/// | 11 | HELLO_WATCOM  | @2   | Just **Non-Resident names** (public exports)
/// +---------------------------+ or unused by module exports
/// 
/// ```
pub(crate) struct NeExecutableLayout {
    pub dos_header: MzHeader,
    pub new_header: NeHeader,
    pub ent_tab: EntryTable,
    pub seg_tab: Vec<NeSegment>,
    pub nres_tab: NonResidentNameTable,
    pub resn_tab: ResidentNameTable,
    pub mod_tab: NeModuleReferencesTable,
    pub imp_tab: Vec<NeSegmentDllImportsTable>
}

impl NeExecutableLayout {
    pub fn get(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let dos_header = MzHeader::read(&mut reader)?;
        if !dos_header.has_valid_magic() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid magic for dos_header"));
        }

        reader.seek(SeekFrom::Start(dos_header.e_lfanew as u64))?;

        if dos_header.e_lfanew == 0_u32 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid e_lfanew for protected-mode executable"));
        }

        let new_header = NeHeader::read(&mut reader)?;
        if  !new_header.is_valid_magic() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid magic for protected-mode executable"));
        }

        // Now we are extremely needed the e_lfanew just because
        // all pointers in Windows-OS/2 header are relative.
        // This is a chance to little compress data to NEAR pointers

        reader.seek(SeekFrom::Start(new_header.e_nres_tab as u64))?;
        let nres_tab = NonResidentNameTable::read(&mut reader)?;

        reader.seek(SeekFrom::Start((new_header.e_resn_tab as u32 + dos_header.e_lfanew) as u64))?;
        let resn_tab = ResidentNameTable::read(&mut reader)?;

        reader.seek(SeekFrom::Start((new_header.e_ent_tab as u32 + dos_header.e_lfanew)as u64))?;
        let ent_table = EntryTable::read(&mut reader, new_header.e_cb_ent)?;

        reader.seek(SeekFrom::Start((new_header.e_mod_tab as u32 + dos_header.e_lfanew) as u64))?;
        let mod_tab = NeModuleReferencesTable::read(&mut reader, new_header.e_cmod)?;


        let mut imp_list = Vec::<NeSegmentDllImportsTable>::new();
        let mut segments = Vec::<NeSegment>::new();
        reader.seek(SeekFrom::Start((dos_header.e_lfanew + new_header.e_seg_tab as u32 ) as u64))?;

        for i in 0..new_header.e_cseg {
            let seg = NeSegment::read(&mut reader, new_header.e_align)?;
            segments.push(seg);
        }

        for (i, s) in segments.iter().enumerate() {
            imp_list.push(NeSegmentDllImportsTable::read(
                &mut reader,
                &s.relocs,
                dos_header.e_lfanew + (new_header.e_imp_tab as u32),
                dos_header.e_lfanew + (new_header.e_mod_tab as u32),
                (i + 1) as i32
            )?);
        }

        let layout = NeExecutableLayout{
            dos_header,
            new_header,
            ent_tab: ent_table,
            nres_tab,
            resn_tab,
            seg_tab: segments,
            mod_tab,
            imp_tab: imp_list
        };

        Ok(layout)
    }
}