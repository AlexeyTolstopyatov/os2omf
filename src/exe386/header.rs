//! This module represents API for extracting common information
//! from Linear (or FLAT) executable. All following data declared in `LinearExecutableHeader`
//! and nested structures (pointers to structures) depends on this executable header position
//!
//! Traditionally IBM and Microsoft flat executables start from `IA-32` real-mode DOS part
//! (means MZ header and following next real-mode application).
//! That's why in current example we need to avoid MZ header and other details.
//! Our goals:
//!  - Skip PC(MS)-DOS header;
//!  - Skip IA-32 application (or DOS stub);
//!  - Get raw file pointer to IA-32 protected-mode exec header.
//!  - Get all IA-32 protected-mode header. (expecting `LE` or `LX`)
//! ```rust
//! use std::fs::File;
//! use std::io::{BufReader, Read, Seek, SeekFrom};
//! use os2omf::exe386::header::LinearExecutableHeader;
//!
//! use std::ptr::read;
//!
//! const NEXT_SIGNATURE_PTR: u64 = 0x3C;
//!
//! let file_str = "<put here path to FLAT executable>";
//! let file_io  = File::open(file_str)?;
//! let mut file_buf = BufReader::new(file_io);
//!
//! file_buf.seek(SeekFrom::Start(NEXT_SIGNATURE_PTR))?; // move to e_lfanew
//! let mut next_ptr_buffer = [0_u8; 4]; // e_lfanew is DWORD typed field
//! file_buf.read_exact(&mut next_ptr_buffer)?;
//!
//!
//! let next_ptr = u64::from_le_bytes(next_ptr_buffer);
//! file_buf.seek(SeekFrom::Start(next_ptr))?;
//!
//! // finally!
//! let exec_flat = LinearExecutableHeader::read(&mut file_buf)?;
//!
//! ```
//!
//! Now we've got the intel protected-mode FLAT header.
//! This pretty simple and mandatory logic, but it demonstrates what we need to give
//! for using this module's API.
//!
//! Sometimes you can find linked dynamic libraries or programs which
//! don't have real-mode application part. (MZ structure is missing),
//! but first file signature is LX.
//! It makes our task easier. We can skip 2 points in list.
//! Then our goals become
//!  - Make sure this signature belongs to IBM FLAT executable.
//!  - Read next whole following data.
//!
use bytemuck::{Pod, Zeroable};
use std::io::{Error, ErrorKind, Read};

pub const LX_MAGIC: u16 = 0x584C;
pub const LX_CIGAM: u16 = 0x4C58;
pub const LE_MAGIC: u16 = 0x455C;
pub const LE_CIGAM: u16 = 0x4C45;
///
/// Linear Executable format is undocumented format
/// From Microsoft Windows and IBM/Microsoft OS/2 till eCOM Station and ArcaOS it was
/// experimental format of program/modules linkage.
///
/// ### LE - Microsoft OS/2 2.0 / Windows VMM
///
/// > Those files mostly contains 16-32 bit code
///
/// All drivers of Windows 3.x ".386" and Windows VMM drivers ".vxd"
/// was LE linked. And OS/2 2.0+ library modules was LE linked. (Unlike of programs)
/// They are contained 16-32 bit code in objects/segments.
///
/// ### LX - IBM OS/2 2.0+ Standard object format
///
/// > Those files are used in modern versions of OS/2 -> ArcaOS.
/// > Mostly contains 32-bit code. But could contain 16-bit code/data objects.
///
/// LX marked executables are "Linear eXecutables". (Open)Watcom
/// compiler and linker uses this format as default for OS/2 OMFs
/// but DOS extenders are LE-linked.
///
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Pod, Zeroable)]
pub struct LinearExecutableHeader {
    pub e32_magic: u16,
    pub e32_border: u8,
    pub e32_worder: u8,
    pub e32_level: u32,
    pub e32_cpu: u16,
    pub e32_os: u16,
    pub e32_ver: u32,
    pub e32_mflags: u32,
    pub e32_mpages: u32,
    pub e32_cs: u32,
    pub e32_eip: u32,
    pub e32_ss: u32,
    pub e32_esp: u32,
    pub e32_pagesize: u32,
    /// Depends on target format:
    ///  - `e32_page_shift` as u32 - LX linked project
    ///  - `e32_cblp` (count bytes at last page) - LE linked project
    pub e32_pageshift_or_lastpage: u32,
    pub e32_fixupsize: u32,
    pub e32_fixupsum: u32,
    pub e32_ldrsize: u32,
    pub e32_ldrsum: u32,
    pub e32_objtab: u32,
    pub e32_objcnt: u32,
    pub e32_objmap: u32,
    pub e32_itermap: u32,
    pub e32_rsrctab: u32,
    pub e32_rsrccnt: u32,
    pub e32_restab: u32,
    pub e32_enttab: u32,
    pub e32_dirtab: u32,
    pub e32_dircnt: u32,
    pub e32_fpagetab: u32,
    pub e32_frectab: u32,
    pub e32_impmod: u32,
    pub e32_impmodcnt: u32,
    pub e32_impproc: u32,
    pub e32_pagesum: u32,
    pub e32_datapage: u32,
    pub e32_preload: u32,
    pub e32_nrestab: u32,
    pub e32_cbnrestab: u32,
    pub e32_nressum: u32,
    pub e32_autodata: u32,
    pub e32_debuginfo: u32,
    pub e32_debuglen: u32,
    pub e32_instpreload: u32,
    pub e32_instdemand: u32,
    pub e32_heapsize: u32,
    /// Available only for LX linked modules
    pub e32_stacksize: u32,
    pub e32_res3: [u8; 8],
}

impl LinearExecutableHeader {
    pub fn read<T: Read>(r: &mut T) -> Result<Self, Error> {
        let mut buf = [0; 184]; // 184+12 = 200
        r.read_exact(&mut buf)?;

        let header: &LinearExecutableHeader = bytemuck::try_from_bytes(&buf)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Unable to cast bytes into header"))?;

        if header.invalid_magic() {
            return Err(Error::new(ErrorKind::InvalidData, format!("Invalid magic 0x{:X}", header.e32_magic)));
        }
        
        if !header.le_byte_ordering() {
            return Err(Error::new(ErrorKind::InvalidData, "Only Little endian linked modules are supported!"))
        }
        
        Ok(*header)
    }
    pub fn external_relocs_stripped(&self) -> bool {
        self.e32_mflags & 0x00000020 != 0
    }
    ///
    /// The setting of this bit in a Linear Executable Module indicates that each
    /// object of the module has a preferred load address specified in the Object
    /// Table Reloc Base Addr. If the module's objects can not be loaded at these
    /// preferred addresses, then the relocation records that have been retained in
    /// the file data will be applied
    ///
    /// In practice if internal relocations stripped -- module still
    /// has fixup records table. But may not contain internal fixups.
    /// But module with internal relocations (relocs_stripped flag not set)
    /// always has internal relocations in `FixupRelocations` table
    /// 
    pub fn internal_relocs_stripped(&self) -> bool {
        self.e32_mflags & 0x00000010 != 0
    }
    pub fn module_type(&self) -> LinearExecutableType {
        if self.e32_mflags & 0x00008000 != 0 {
            return LinearExecutableType::DLL;
        }
        if self.e32_mflags & 0x00020000 != 0 {
            return LinearExecutableType::PDD;
        }
        if self.e32_mflags & 0x00028000 != 0 {
            return LinearExecutableType::VDD;
        }
        if self.e32_mflags & 0x00030000 != 0 {
            return LinearExecutableType::DLD;
        }
        LinearExecutableType::EXE
    }
    ///
    /// Be carefully: Only little endian-ordered files are supported!
    ///
    pub fn le_byte_ordering(&self) -> bool {
        if self.e32_border == 0 && self.e32_worder == 0 {
            return true;
        }
        false
    }
    /// Matches `e32_magic` with program-constants
    /// declared higher in `exe386::header`
    pub fn invalid_magic(&self) -> bool {
        match self.e32_magic {
            LX_MAGIC | LX_CIGAM => true,
            LE_MAGIC | LE_CIGAM => true,
            _ => false,
        }
    }
}
#[repr(u16)]
pub enum CPU {
    /// Intel 286 and higher
    I286 = 0x0001,
    /// Intel 386 and higher
    I386 = 0x0002,
    /// Intel 486 and higher
    I486 = 0x0003,
}
pub enum OS {
    Unknown = 0,
    /// OS/2 2.0+
    Os2v2 = 0x0001,
    /// Windows without 32-bit support
    Windows286 = 0x0002,
    /// DOS 4.0+
    Dos4 = 0x0003,
    /// Windows with support of 32-bit code execution
    /// Be carefully: not `Win32s`. Win32s is a "Win32-subsystem" codename for Windows 3x
    Windows386 = 0x0004,
    PersonalityNeural = 0x0005,
}
/// Possible declared by IBM manual types of loadable modules
#[repr(u32)]
pub enum LinearExecutableType {
    /// Executable
    EXE = 0x00000000,
    /// Dynamically linked library
    DLL = 0x00008000,
    /// Physical Device Driver
    PDD = 0x00020000,
    /// Virtual Device Driver
    VDD = 0x00028000,
    /// Dynamically linked Driver
    DLD = 0x00030000,
}