use bytemuck::{Pod, Zeroable};
///
/// Windows Virtual xxx Drivers appears in traditional
/// understanding appears in Windows 3x (NOT Windows 1.x)
/// and was a dangerous objects of OS.
///
/// MS-DOS Mz-executables was an applications what runs
/// in IA-32 real-mode and strongly requires physical devices
/// for themselves to work correctly.
/// 
/// Windows virtual device drivers are emulates work
/// of physical devices and gives all processed signals 
/// to VMM. Only VMM has rights to call physical devices.
///  
#[repr(C, packed(1))]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Pod, Zeroable)]
pub struct VxDHeader {
    pub e32_win_rsrc_offset: u32,
    pub e32_win_rsrc_size: u32,
    pub e32_device_id: u16,
    pub e32_ddk_major: u16,
    pub e32_ddk_minor: u16,
}

///
/// This structure is a marker of Windows VMM virtual drivers
/// Mostly embeds in VXD drivers built using
/// Windows 95/98-ME Driver Development kit.
/// They are called "Windows VMM drivers" because Windows 9x
/// uses "Virtual Machine Manager" as a hypervisor before MS-DOS
/// loads and "died" into memory.
///
/// Windows 3x drivers (*.386 files) aren't have
/// this structure and nested VERSION_INFO, FIXED_STRING_INFO
/// resources. Pointer to the structure will be NULL (0).
///
#[repr(C, packed(1))]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Pod, Zeroable)]
pub struct VxDRsrcHeader {
    pub rsrc_type: u8,
    pub rsrc_name: u8,
    pub rsrc_ordinal: u16,
    pub rsrc_flags: u16,
    pub rsrc_length: u16,
    // next following types are standard resource scripts
    // (I suppose they are really compiled as .RES
    // and embedded into Windows drivers)
    // pub rsrc_version_info: Win32VersionInfo
}