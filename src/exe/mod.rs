//! This module represents API for extracting information of i8080/i8086 programs
//! written and linked for MS-DOS Windows OS/2 and other platforms which are having
//! DOS trace.
//!
//! Programs what started with `MZ` were so close to hardware because
//! MS-DOS 2.0...4.0 ran into IA-32 real-mode. In modern applications or libraries
//! this called DOS header and following next real-mode application called "DOS Stub".
//!
//! Extracting information from DOS executable is easy task
//! but not all knows what follows by the DOS header.
//! ```rust
//! use std::fs::File;
//! use std::io::BufReader;
//! use os2omf::exe::MzHeader;
//! use os2omf::exe::reltab::MzRelocationTable;
//!
//! let file_str = "<put here any exe filepath>";
//! let file_io = File::open(file_str)?;
//! let mut file_buf = BufReader::new(file_io);
//!
//! let dos_header = MzHeader::read(&mut file_buf)?;
//! let dos_relocations = MzRelocationTable::read(&mut file_buf, &dos_header)?;
//! ```    
//! 
//! If you look into filled MZ header of any Windows executable you will see
//! once interesting thing: "`e_lfarlc` always set 0x40". This rule was since
//! Windows 1.0 was released. Applications of Windows 1.x were already IA-32 protected-mode programs.
//! 
//! The rule which determines next protected-mode program location
//! exists too, but I can't follow it. In practice, it would be better if you
//! will agree with long jumps between DOS real-mode sections of data and code and
//! protected-mode sections of data and code.
//! If you see anomaly long jump at `e_lfanew` it may be
//!  - DOS Extender's runtime instead of DOS stub (e.g. DOS4GW/DOS32a/Watcom);
//!  - Windows386 self-executable archive (W3/W4);
//!  - Invalid pointer.
//! 
//! Use this when you are deep dive into retro software.
pub mod reltab;

use crate::exe::reltab::MzRelocationTable;
use bytemuck::{Pod, Zeroable};
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::{BufReader, ErrorKind};

pub const E_MAGIC: u16 = 0x5a4d;
pub const E_CIGAM: u16 = 0x4d5a;
pub const E_LFARLC: u16 = 0x40;

pub struct MzExecutableLayout {
    pub header: MzHeader,
    pub relocs: MzRelocationTable
}

impl MzExecutableLayout {
    pub fn get(file_name: &str) -> Result<Self, io::Error> {
        let file = File::open(file_name)?;
        let mut reader = BufReader::new(file);
        let result_header = MzHeader::read(&mut reader);

        let header = match result_header {
            Ok(header) => header,
            Err(e) => return Err(e)
        };

        let result_relocs = MzRelocationTable::read(&mut reader, &header);

        let relocs = match result_relocs {
            Ok(r) => r,
            Err(e) => return Err(e)
        };

        Ok(Self {
            header,
            relocs
        })
    }
}

///
/// Mark Zbikowski header of DOS programs
///
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct MzHeader {
    /// MZ Header signature
    pub e_magic: u16,
    /// Bytes on last page of file
    pub e_cblp: u16,
    /// Pages in file
    pub e_cp: u16,
    /// Relocations
    pub e_crlc: u16,
    /// Size of header in paragraphs
    pub e_cparhdr: u16,
    /// Minimum extra paragraphs needed
    pub e_minalloc: u16,
    /// Maximum extra paragraphs needed
    pub e_maxalloc: u16,
    /// Initial (relative) SS value
    pub e_ss: u16,
    /// Initial SP value
    pub e_sp: u16,
    /// Checksum
    pub e_crc: u16,
    /// Initial IP value
    pub e_ip: u16,
    /// Initial (relative) CS value
    pub e_cs: u16,
    /// File address of relocation table
    pub e_lfarlc: u16,
    /// Overlay number
    pub e_ovno: u16,
    /// Reserved words
    pub e_res: [u16; 4],
    /// OEM identifier (for e_oeminfo)
    pub e_oemid: u16,
    /// OEM information; e_oemid specific
    pub e_oeminfo: u16,
    /// Reserved words
    pub e_res2: [u16; 10],
    /// Offset to extended header
    pub e_lfanew: u32,
}
impl MzHeader {
    ///
    /// Fills header from target file using prepared
    /// binary reader instance.
    ///
    pub fn read<TRead: Read>(r: &mut TRead) -> io::Result<Self> {
        let mut buf = [0; 0x40];
        r.read_exact(&mut buf)?;

        let header: MzHeader = bytemuck::cast(buf);

        if !header.has_valid_magic() {
            return Err(io::Error::new(ErrorKind::InvalidData, "Invalid DOS header"))
        }

        Ok(header)
    }
    ///
    /// Tries check out signature of PC-DOS executable
    ///
    pub fn has_valid_magic(&self) -> bool {
        match self.e_magic {
            E_CIGAM => true,
            E_MAGIC => true,
            _ => false,
        }
    }
    ///
    /// Tries to validate checksum set in the MZ header
    ///
    pub fn has_valid_crc(&self) -> bool {
        let mut pos: usize = 0;
        let mut sum: u16 = 0;

        let buffer = bytemuck::bytes_of(&self.e_crc);

        while pos < buffer.len() {
            // iterate each buffer element
            let word: [u8; 2] = [buffer[pos], *buffer.get(pos + 1).unwrap_or(&0)];
            let word: u16 = u16::from_le_bytes(word);
            sum = sum.wrapping_add(word);
            pos += 2;
        }

        match sum {
            0 => true,
            _ => false,
        }
    }
    ///
    /// For some reason since the NE (New segmented executables)
    /// the MZ relocations table pointer always set at 0x40 absolute offset
    /// by default.
    /// Without some extern reason pointer to the MZ relocations table
    /// not changes.
    ///
    pub fn has_default_rlcptr(&self) -> bool {
        self.e_lfarlc == E_LFARLC
    }
}
