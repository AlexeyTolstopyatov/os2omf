mod reltab;

use bytemuck::{Pod, Zeroable};
use std::{
    io::{self, Read},
    u8,
};
use std::io::ErrorKind;

pub const E_MAGIC: u16 = 0x5a4d;
pub const E_CIGAM: u16 = 0x4d5a;
pub const E_LFARLC: u16 = 0x40;
///
/// Mark Zbikowski header of DOS programs
///
/// transparent -> StructLayout=Explicit
/// C           -> StructLayout=Sequential
/// packed      -> Pack = 1
///
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub(crate) struct MzHeader {
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
    /// @returns filled MZ executable header
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
    /// Tries check out signature of PC-DOS
    /// x86 real-mode executable
    ///
    /// @return: Optional value with Some(unit) or an io::Error prepared instance
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
    /// @returns: Some(unit) or prepared io::Error instance
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
    /// @returns: boolean flag of "linker set relocations pointer".
    ///
    pub fn has_default_rlcptr(&self) -> bool {
        self.e_lfarlc == E_LFARLC
    }
}
