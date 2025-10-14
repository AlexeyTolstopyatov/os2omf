use std::io::{self, Read};
use bytemuck::{Pod, Zeroable};

use crate::exe286;

///
/// OS/2 & Windows file header definitions
/// 
#[repr(C, packed(1))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Zeroable, Pod)]
pub(crate) struct NeHeader {
    pub e_magic: u16,
    pub e_lnkver: u8,
    pub e_lnkrev: u8,
    pub e_enttab: u16,
    pub e_cbentt: u16,
    pub e_crc: u32,
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

///
/// Interface of New Executable header
/// 
impl NeHeader {
    pub fn read<TR: Read>(r: &mut TR) -> io::Result<Self> {
        let mut buf = [0; 0x40];
        r.read_exact(&mut buf)?;
        
        return Ok(bytemuck::cast(buf));
    }
    pub fn check_magic(&self) -> io::Result<()> {
        return match self.e_magic {
            exe286::NE_CIGAM => Ok(()),
            exe286::NE_MAGIC => Ok(()),
            _ => Err(
                io::Error::new(io::ErrorKind::InvalidData, "Bad magic number)")
            )
        };
    }
}