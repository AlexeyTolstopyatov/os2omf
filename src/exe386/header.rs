use bytemuck::{Pod, Zeroable};
use std::io::{Error, ErrorKind, Read};

pub const LX_MAGIC: u16 = 0x584c;
pub const LX_CIGAM: u16 = 0x4c58;
pub const LE_MAGIC: u16 = 0x455c;
pub const LE_CIGAM: u16 = 0x4c45;
///
/// Linear Executable format is undocumented format
/// From Microsoft Windows and IBM OS/2 till eCOM Station and ArcaOS it was
/// experimental format of program/modules linkage.
///
/// ### LE - OS/2 2.0 and Windows VMM era
///
/// > Those files mostly contains 16-32 bit code
///
/// LE marked executables are "Linear Executables". Nobody knows
/// what specification was first, so I feel - LE format was first.
/// All drivers of Windows 3.x ".386" and Windows VMM drivers ".vxd"
/// was LE linked. And OS/2 2.0+ DOSCA11S.DLL library module was LE linked.
/// They are contained 16-32 bit code in objects/segments.
///
/// ### LX - OS/2 3.0+ Standard object format
///
/// > Those files are used in modern versions of OS/2 -> ArcaOS.
/// > Mostly contains 32-bit code. But can be semi 32-bit too.
///
/// LX marked executables are "Linear eXecutables". (Open)Watcom
/// compiler and linker uses this format as default for OS/2 OMFs
/// but DOS extenders are LE-linked.
///
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Pod, Zeroable)]
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
    pub e32_pageshift: u32,
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
    pub e32_stacksize: u32,
    pub e32_res3: [u8; 8],
}
impl LinearExecutableHeader {
    pub fn read<T: Read>(r: &mut T) -> Result<LinearExecutableHeader, Error> {
        let mut buf = [0; 184]; // 184+12 = 200
        r.read_exact(&mut buf)?;
        
        let header = bytemuck::try_from_bytes(&buf)
            .map_err(|e| Error::new(ErrorKind::InvalidData, "Unable to cast bytes into header"))?;
        
        Ok(*header)
    }
}
#[repr(u16)]
pub enum CPU {
    I286 = 0x0001,
    I386 = 0x0002,
    I486 = 0x0003,
}
pub enum OS {
    Unknown = 0,
    Os2 = 0x0001,
    Windows = 0x0002,
    Dos4 = 0x0003,
    Windows386 = 0x0004,
    PersonalityNeural = 0x0005,
}
pub struct LinearExecutableFlags {
    flags: u32,
    fmt_lvl: u32,
    bo: u8,
    wo: u8,
    major_ver: u16,
    minor_ver: u16,
}
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
impl LinearExecutableFlags {
    pub fn from(
        flags: u32,
        byte_order: u8,
        word_order: u8,
        fmt_lvl: u32,
        ver: u32,
    ) -> LinearExecutableFlags {
        let major_ver = ver >> 16;
        let minor_ver = ver & 0xffff;
        LinearExecutableFlags {
            flags,
            bo: byte_order,
            wo: word_order,
            fmt_lvl,
            major_ver: major_ver as u16,
            minor_ver: minor_ver as u16,
        }
    }
    pub fn external_relocs_stripped(&self) -> bool {
        self.flags & 0x00000020 != 0
    }
    /// 
    /// The setting of this bit in a Linear Executable Module indicates that each
    /// object of the module has a preferred load address specified in the Object
    /// Table Reloc Base Addr. If the module's objects can not be loaded at these
    /// preferred addresses, then the relocation records that have been retained in
    /// the file data will be applied
    /// 
    /// In practice if internal relocations stripped -- module still
    /// having fixup records table. But doesn't contain internal fixups.
    /// 
    pub fn internal_relocs_stripped(&self) -> bool {
        self.flags & 0x00000010 != 0
    }
    pub fn module_type(&self) -> LinearExecutableType {
        if self.flags & 0x00008000 != 0 {
            return LinearExecutableType::DLL;
        }
        if self.flags & 0x00020000 != 0 {
            return LinearExecutableType::PDD;
        }
        if self.flags & 0x00028000 != 0 {
            return LinearExecutableType::VDD;
        }
        if self.flags & 0x00030000 != 0 {
            return LinearExecutableType::DLD;
        }
        LinearExecutableType::EXE
    }
}
//
// Mostly reverse engineering of LE linked binaries bases
// on the IBM documents about LX format. Therefore,
// I'll base on last revision of "IBM Object Module | Format Linear eXecutable".pdf
//
