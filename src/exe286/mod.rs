use crate::exe286::enttab::EntryTable;
use crate::exe286::header::NeHeader;
use crate::exe286::segrelocs::RelocationTable;
use crate::exe286::segtab::NeSegment;
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
/// |             | always eq.  |     | Absolute offset what holds in e_lfanew
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
/// | .CODE segment 1 [relocs..]| Segments Table of New Executable contains not just
/// | .CODE segment 2 [relocs..]| segments data of length and positions. For each segment
/// | .DATA segment 3 []        | in table if flags byte mask contains SEG_HASRELOC (0x0100)
/// |                   +-------+ exists following next array of relocations.
/// +-------------------+       |
/// |        padding            | 
/// +----+----+-------+----+----+
/// | 00 | 03 | "GDI" | 03 | MSG| Importing modules strings
/// +----+----+-------+----+----+ 
/// | 09 | FATALEXIT  |         |    + Just Resident names (private exports)
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
/// | 11 | HELLO_WATCOM  | @2   | <-- Just Non-Resident names (public exports)
/// +---------------------------+     or unused by module exports
/// 
// /// ```
/// 
pub(crate) struct NeExecutableLayout {
    pub dos_header: Box<MzHeader>,
    pub win_header: Box<NeHeader>,
    pub ent_table: EntryTable,
    pub seg_table: Vec<NeSegment>,
    pub seg_reloc_table: Vec<RelocationTable>,
    
}