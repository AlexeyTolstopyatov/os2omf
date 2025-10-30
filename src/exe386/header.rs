use bytemuck::AnyBitPattern;

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
#[repr(C, packed(1))]
#[derive(Copy, Clone, PartialEq, Eq, AnyBitPattern)]
pub struct Os2ModuleHeader {
    pub e32_magic: u16,
    pub e32_border: u8,
    pub e32_worder: u8,
    pub e32_level: u32,
    pub e32_cpu: u16,
    pub e32_os: u16,
    pub e32_ver: u32,
    pub e32_mflags: u32,
    pub e32_mpages: u32,
    pub e32_startobj: u32,
    pub e32_eip: u32,
    pub e32_stackobj: u32,
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
    pub e32_res3: [u8; 20],
}
//
// Mostly reverse engineering of LE linked binaries bases
// on the IBM documents about LX format. Therefore,
// I'll base on last revision of "IBM Object Module | Format Linear eXecutable".pdf
//
