use std::io::{self, Read};
use bytemuck::{Pod, Zeroable};

use crate::exe286;

///
/// OS/2 & Windows file header definitions
/// 
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Zeroable, Pod)]
pub(crate) struct NeHeader {
    pub e_magic: u16,
    pub e_lnkver: u8,
    pub e_lnkrev: u8,
    pub e_enttab: u16,
    pub e_cbentt: u16,
    pub e_crc: u32,
    /// ### Definitions of e_flags byte-mask
    /// ```
    /// Flags of module represent byte-mask.
    /// Byte-mask defines 2 big groups of bytes: Program-Flags and Application-Flags
    /// In hexadecimal view it looks like 4 columns:
    ///     0   0   0   0<-- [Program]
    ///     |   |   +------- [Application]
    ///     |   +------------[Application]
    ///     +----------------[Program] (reserved scope) LINKERR / NONCONFORM / DLL
    ///```
    pub e_flags: u16,
    pub e_autodata: u16,
    pub e_heap: u16,
    pub e_stack: u16,
    pub e_ip: u16,
    pub e_cs: u16,
    pub e_sp: u16,
    pub e_ss: u16,
    pub e_cseg: u16,
    pub e_cmod: u16,
    pub e_cbnres: u32,
    pub e_segtab: u16,
    pub e_rsrctab: u16,
    pub e_resntab: u16,
    pub e_modtab: u16,
    pub e_imptab: u16,
    pub e_nrestab: u16,
    pub e_cmovent: u16,
    pub e_align: u16,
    pub e_crsrc: u16,
    pub e_os: u8,
    pub e_flagsothers: u16,
    // OS/2 Header starts here
    pub e_retthunk: u16,
    pub e_segref_thunk: u16,
    pub e_swap: u16,
    pub e_winver: u8,
    pub e_winrev: u8
}
#[repr(u16)]
pub enum CPU {
    Undefined = 0,
    I8086 = 0x0004,
    I286 = 0x0005,
    I386 = 0x0006,
    I8087 = 0x0007,
}
impl CPU {
    pub fn get_from(flags: u16) -> CPU {
        match flags {
            0x0004 => CPU::I8086,
            0x0005 => CPU::I286,
            0x0006 => CPU::I386,
            0x0007 => CPU::I8087,
            _ => CPU::Undefined
        }
    }
}
///
/// Interface of New Executable header
/// 
impl NeHeader {
    pub fn read<TRead: Read>(r: &mut TRead) -> io::Result<Self> {
        let mut buf = [0; 0x40];
        r.read_exact(&mut buf)?;

        Ok(bytemuck::cast(buf))
    }
    /// Returns the check magic of [`NeHeader`].
    /// 
    /// # Errors
    /// This function will return an error if header contains
    /// unexpected magic number.
    pub fn check_magic(&self) -> io::Result<()> {
        match self.e_magic {
            exe286::NE_CIGAM => Ok(()),
            exe286::NE_MAGIC => Ok(()),
            _ => Err(
                io::Error::new(io::ErrorKind::InvalidData, "Bad magic number)")
            )
        }
    }
}