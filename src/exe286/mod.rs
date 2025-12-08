//! This module represents API for extracting all nested information
//! from new segmented executable file. This format was a first format
//! of Intel protected-mode executables. Microsoft made it pretty simple but impressive.
//!
//! All dynamic linking features and run-time imports are growing from here.
//! Nested resources and windowing applications are growing from here too.
//! All calling conventions and understandable of DOS API are growing from here too.
//! And all of it was in 16-bit executables loaded in segmented memory.
//!
//! Files linked as NE executables are appearing in Windows 1.x till 3.x,
//! IBM OS/2 1.x, multitasking MS-DOS 4.x, and other DOS editions.
//! Let's extract all data and symbols from those files:
//! ```rust
//! use os2omf::exe286::NewExecutableLayout;
//!
//! let file_str = "put here Windows 3.1 app/dll path";
//! let layout = NewExecutableLayout::get(file_str)?;
//!
//! ```
//! That's all. `layout` contains all extracted and processed data
//! of nested structures what follows by the header.
use crate::exe::MzHeader;
use crate::exe286::enttab::EntryTable;
use crate::exe286::header::NewExecutableHeader;
use crate::exe286::modtab::ModuleReferencesTable;
use crate::exe286::nrestab::NonResidentNameTable;
use crate::exe286::resntab::ResidentNameTable;
use crate::exe286::segtab::{ImportsTable, Segment};
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Seek;
use std::io::SeekFrom;

pub const NE_MAGIC: u16 = 0x454e;
pub const NE_CIGAM: u16 = 0x4e45;

// connect modules "files" here
pub mod enttab;
pub mod header;
pub mod modtab;
pub mod nrestab;
pub mod resntab;
pub mod segrelocs;
pub mod segtab;
/// ### Segmented New Executable Layout
/// Every segmented OS/2-Windows executable is a book with specific data inside
/// This book traditionally has table of content
/// Main regions of this book is a segments like sections in PE32/+ or ELF32/64 files
///
/// ```
/// +----+---+--------+---------+
/// | MZ |   |e_lfarlc|e_lfanew ------+
/// +----+---+----|---+---------+     |
/// |             | always eq.  |     | **Absolute offset** what holds in e_lfanew
/// |             | 0x40 (64)   |     | is an raw file pointer to next structure
/// |[      ]<----+             |     |
/// |                           |     | That's why it calls e_lfanew.
/// |                     +-----+     | "long file address (of) new executable"
/// |                     | NE  <-----+ <--+
/// +---+---+---+---+---+---+---+          | New Executable Header.
/// |   |   |   |   |   |   |   |          | Main parameter here is raw position
/// +---+---+---+---+---+---+---+          | of new executable header.
/// |   |   |   |   |   |   |   |          | All pointers in here are relative.
/// +---------------------------+<---------+
/// |        padding            |
/// +---------------------------+
/// |                           | **Segments Table** of New Executable contains not just
/// |      SEGMENTS TABLE       | segments data of length and positions. For each segment
/// |                           | in table if flags byte mask contains SEG_HASRELOC (0x0100)
/// |---------------------------+ exists following next array of relocations.
/// |        padding            |
/// +---------------------------+
/// |  MODULE REFERENCES TABLE  | Pointers (indexes) of import .DLL/EXE Strings
/// +----+----+-------+----+----+
/// | 00 | IMPORT MODULES TABLE | **Importing modules** strings
/// +----+----+-------+----+----+
/// |   RESIDENT NAMES TABLE    | Exporting functions which kept in memory
/// +---------------------------+ while module loaded
/// |                           |
/// |      RESOURCES TABLE      |
/// |                           |
/// |---------------------------+
/// |                           | <-- EntryPoints Table
/// |       ENTRY TABLE         |     Main table for all exports in module
/// |                 +---------|     holds positions and to every entry
/// +-----------------+         |     in all registered segments in file.
/// |                           |
/// |       DATA AND CODE       |
/// |  (with paddings between)  |
/// |                           |
/// +---------------------------+
/// |  NONRESIDENT NAMES TABLE  | Exporting functions which unused by module
/// +---------------------------+
///
/// ```

pub struct NewExecutableLayout {
    pub dos_header: MzHeader,
    pub new_header: NewExecutableHeader,
    pub ent_tab: EntryTable,
    pub seg_tab: Vec<Segment>,
    pub nres_tab: NonResidentNameTable,
    pub resn_tab: ResidentNameTable,
    pub mod_tab: ModuleReferencesTable,
    pub imp_tab: Vec<ImportsTable>,
}

impl NewExecutableLayout {
    pub fn get(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let dos_header = MzHeader::read(&mut reader)?;
        if !dos_header.has_valid_magic() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not a valid DOS header",
            ));
        }

        reader.seek(SeekFrom::Start(dos_header.e_lfanew as u64))?;

        if dos_header.e_lfanew == 0_u32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not protected mode executable",
            ));
        }

        let offset = |ptr: u16| ptr as u64 + dos_header.e_lfanew as u64;

        let new_header = NewExecutableHeader::read(&mut reader, dos_header.e_lfanew)?;
        if !new_header.is_valid_magic() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid magic for protected-mode executable",
            ));
        }
        // Now we are extremely needed the e_lfanew just because
        // all pointers in Windows-OS/2 header are relative.
        // This is a chance to little compress data to NEAR pointers
        let nres_tab = NonResidentNameTable::read(&mut reader, new_header.e_nres_tab)?;
        let resn_tab = ResidentNameTable::read(&mut reader, offset(new_header.e_resn_tab))?;
        let ent_table = EntryTable::read(
            &mut reader,
            offset(new_header.e_ent_tab),
            new_header.e_cb_ent,
        )?;
        let mod_tab = ModuleReferencesTable::read(
            &mut reader,
            offset(new_header.e_mod_tab),
            new_header.e_cmod,
        )?;
        let mut imp_list = Vec::<ImportsTable>::new();
        let mut segments = Vec::<Segment>::new();

        reader.seek(SeekFrom::Start(offset(new_header.e_seg_tab)))?;

        for _ in 0..new_header.e_cseg {
            let seg = Segment::read(&mut reader, new_header.e_align)?;
            segments.push(seg);
        }

        for (i, s) in segments.iter().enumerate() {
            imp_list.push(ImportsTable::read(
                &mut reader,
                &s.relocs,
                offset(new_header.e_imp_tab) as u32,
                offset(new_header.e_mod_tab) as u32,
                (i + 1) as i32,
            )?);
        }

        let layout = Self {
            dos_header,
            new_header,
            ent_tab: ent_table,
            nres_tab,
            resn_tab,
            seg_tab: segments,
            mod_tab,
            imp_tab: imp_list,
        };

        Ok(layout)
    }
}
