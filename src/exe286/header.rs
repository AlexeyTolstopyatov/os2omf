//!
use bytemuck::{Pod, Zeroable};
use std::io::{self, Read, Seek, SeekFrom};

use crate::exe286;

///
/// OS/2 & Windows file header definitions
///
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
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
    pub fn from(flags: u16) -> CPU {
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
    pub fn is_valid_magic(&self) -> bool {
        match u16::from_le_bytes(self.e_magic) {
            exe286::NE_CIGAM => true,
            exe286::NE_MAGIC => true,
            _ => false,
        }
    }
    pub fn module_flags(&self) -> ModuleFlags {
        ModuleFlags {
            linkage_errors: self.e_flags & 0x8000 != 0,
            library_module: self.e_flags & 0x0002 != 0,
            protected_mode_only: self.e_flags & 0x0008 != 0,
            data_segment: DataSegment::from(self.e_flags),
            app_flags: ModuleWindowing::from(self.e_flags),
        }
    }
}

/// One `WORD` field `e_flags` contains 2 categories
/// named "Program Flags" and "Application Flags". This information
/// applies since Windows 3.1 and SDK was released.
///
/// High and Low hexadecimal digits belongs to Program-flags
/// ```
/// // 0x0000
/// //   ^  ^
/// //   are program flags byte-mask
/// ```
/// They are contains information about target CPU and special
/// characteristics (e.g. single `.DATA` segment or DS is missing at all)
///
/// Digits between describes module as OS Application. (e.g. "App uses Win16 API")
///
/// ```
/// // 0x0000
/// //    ^^
/// //    are application flags byte-mask
/// ```
///
#[derive(Debug, Clone)]
pub struct ModuleFlags {
    /// 
    library_module: bool,
    data_segment: DataSegment,
    app_flags: ModuleWindowing,
    linkage_errors: bool,
    protected_mode_only: bool,
}
#[derive(Debug, Clone)]
enum DataSegment {
    /// Data segment is missing
    No = 0x0000,
    /// Shared among processes (application instances)
    Single = 0x0001,
    /// For each application instances will be made
    /// new data segment.
    Multiple = 0x0002,
}
#[derive(Debug, Clone)]
enum ModuleWindowing {
    /// Module doesn't use Windows PM API
    None = 0x0000,
    /// Application runs in "full-screen" mode
    FullScreen = 0x0010,
    /// Application could run with Windows PM API
    CompatWinAPI = 0x0020,
    /// Application requires installed Windows PM API
    UseWinAPI = 0x0030,
}
impl ModuleWindowing {
    pub fn from(f: u16) -> Self {
        match f & 0x00FF {
            0x0001 => ModuleWindowing::FullScreen,
            0x0002 => ModuleWindowing::CompatWinAPI,
            0x0003 => ModuleWindowing::UseWinAPI,
            _ => ModuleWindowing::None,
        }
    }
}
impl DataSegment {
    pub fn from(f: u16) -> Self {
        match f & 0x0000 {
            1 => DataSegment::Single,
            2 => DataSegment::Multiple,
            _ => DataSegment::No,
        }
    }
}
