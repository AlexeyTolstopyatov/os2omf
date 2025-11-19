use bytemuck::{Pod, Zeroable};
use std::io::{self, Read, Seek, SeekFrom};

use crate::exe286;

///
/// OS/2 & Windows file header definitions
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct NewExecutableHeader {
    pub e_magic: [u8; 2],
    pub e_link_maj: u8,
    pub e_link_min: u8,
    pub e_ent_tab: u16,
    pub e_cb_ent: u16,
    pub e_load_crc: u32,
    pub e_flags: u16,
    pub e_autodata: u16,
    pub e_heap: u16,
    pub e_stack: u16,
    pub e_csip: u32,
    pub e_sssp: u32,
    pub e_cseg: u16,
    pub e_cmod: u16,
    pub e_cbnres: u16,
    pub e_seg_tab: u16,
    pub e_rsrc_tab: u16,
    pub e_resn_tab: u16,
    pub e_mod_tab: u16,
    pub e_imp_tab: u16,
    pub e_nres_tab: u32,
    pub e_cmov_ent: u16,
    pub e_align: u16,
    pub e_crsrc: u16,
    pub e_os: u8,
    pub e_flag_others: u8,
    pub e_ret_thunk: u16,    // <-- offset
    pub e_segref_thunk: u16, // <-- segment reference thunk offset
    pub min_code_swap: u16,
    pub expected_win_ver: [u8; 2],
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
            _ => CPU::Undefined,
        }
    }
}
///
/// Interface of New Executable header
///
impl NewExecutableHeader {
    pub fn read<TRead: Read + Seek>(r: &mut TRead, e_lfanew: u32) -> io::Result<Self> {
        r.seek(SeekFrom::Start(e_lfanew as u64))?;

        let mut buf = [0; 0x40];
        r.read_exact(&mut buf)?;

        Ok(bytemuck::cast(buf))
    }
    /// Returns the check magic of [`NewExecutableHeader`].
    ///
    /// # Errors
    /// This function will return an error if header contains
    /// unexpected magic number.
    pub(crate) fn is_valid_magic(&self) -> bool {
        match u16::from_le_bytes(self.e_magic) {
            exe286::NE_CIGAM => true,
            exe286::NE_MAGIC => true,
            _ => false,
        }
    }
    pub(crate) fn get_flags(&self) -> Vec<String> {
        let mut flags = Vec::new();
        flags
    }
}
