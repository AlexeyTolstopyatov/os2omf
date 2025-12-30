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
pub enum OS {
    /// None or Any
    Unknown = 0,
    /// OS/2 1x versions and usage of I286 instructions
    Os2 = 1,
    /// Windows 1.x-3x and usage of I286 instructions
    Windows286 = 2,
    /// European MS-DOS 4.0
    /// (in different words: Multitasking MS-DOS)
    Dos4 = 3,
    /// Windows 1.x-3x and usage of I386 instructions
    Windows386 = 4
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
            non_conforming: self.e_flag_others & 0x0040 != 0,
            image_error: self.e_flag_others & 0x0020 != 0,
        }
    }

    pub fn other_os2_flags(&self) -> ModuleOs2Flags {
        ModuleOs2Flags {
            long_names_support: self.e_flag_others & 0x0001 != 0,
            os2_protected_mode: self.e_flag_others & 0x0002 != 0,
            proportional_fonts: self.e_flag_others & 0x0004 != 0,
            gangload_area: self.e_flag_others & 0x0008 != 0,
        }
    }

    pub fn other_windows_flags(&self) -> ModuleWindowsFlags {
        ModuleWindowsFlags {
            win3x_protected_mode: self.e_flag_others & 0x0002 != 0,
            proportional_fonts: self.e_flag_others & 0x0004 != 0,
            fastload_area: self.e_flag_others & 0x0008 != 0,
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
    /// Library module.
    ///  - The `SS:SP` information is invalid,
    ///  - The `CS:IP` points to an initialization procedure that is called
    /// with `AX` register equal to the module handle.
    ///  - DS is set to the library's data segment if the
    /// SINGLEDATA flag is set. otherwise, DS is set
    /// to the caller's data segment.
    ///
    /// The initialization procedure must perform a `FAR`-return to the caller,
    /// with `AX` _not equal to zero to indicate success_, or `AX` _equal to zero
    /// to indicate failure to initialize._
    ///
    /// A program or DLL can only contain dynamic links to executable files
    /// that have this library module flag set. one program cannot dynamic-link to another program.
    pub library_module: bool,
    /// `.DATA` segment kind of the target
    pub data_segment: DataSegment,
    /// Errors in image (maybe some of the structures might be corrupted)
    pub image_error: bool,
    /// Intel specific value: see "NonConforming image x86"
    pub non_conforming: bool,
    /// Errors detected at link time, module will not load.
    pub linkage_errors: bool,
    /// This flag set if it would be better to run module at `i286` and higher
    /// (this flag not belongs to OS/2)
    pub protected_mode_only: bool,
}
#[derive(Debug, Clone)]
pub enum DataSegment {
    /// Data segment is missing
    No = 0x0000,
    /// Shared among processes (application instances)
    Single = 0x0001,
    /// For each application instances will be made
    /// new data segment.
    Multiple = 0x0002,
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
///
/// If application runs under OS/2 1.x versions
/// The field `e_flagothers` defines like this byte-mask
///
/// I'm not sure how the Win-OS/2 subsystem works with it,
/// but Windows 3.10 defines this field different. (see [ModuleWindowsFlags])
///
/// It would be better if `e_flagothers` byte-mask reinterprets like this structure
/// in the [OS::Os2] case.
///
pub struct ModuleOs2Flags {
    pub os2_protected_mode: bool,
    pub proportional_fonts: bool,
    pub long_names_support: bool,
    pub gangload_area: bool
}
///
/// If application marked as [OS::Windows286] or [OS::Windows386]
/// We can reinterpret `e_flagothers` byte-mask like this.
///
/// This list of flags came with Windows 3.10 SDK.
pub struct ModuleWindowsFlags {
    pub win3x_protected_mode: bool,
    pub proportional_fonts: bool,
    pub fastload_area: bool,
}