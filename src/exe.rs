use bytemuck::{Pod, Zeroable};
use std::{io::{self, Read}, u8};

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
#[repr(C, packed)] 
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
struct MzHeader {
    // DOS header
    pub e_magic: u16,
    pub e_cp: u16,
    pub e_cblp: u16,
    pub e_relc: u16,
    pub e_cphdr: u16,
    pub e_minpar: u16,
    pub e_maxpar: u16,
    pub e_ss: u16,
    pub e_sp: u16,
    pub e_crc: u32,
    pub e_ip: u16,
    pub e_cs: u16,
    pub e_lfarlc: u16,
    pub e_ovno: u16,
    pub e_res_1: [u16; 4], // <-- _'ll become [0,0,0,0]
    pub e_oemid: u16,
    pub e_oeminfo: u16,
    pub e_res_2: [u16; 10],
    pub e_lfanew: u32,
}

impl MzHeader {
    ///
    /// Fills header from target file using prepared 
    /// binary reader instance.
    /// 
    /// @returns filled MZ executable header
    /// 
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut buf = [0; 0x40];
        r.read_exact(&mut buf)?;

        return Ok(bytemuck::cast(buf));
    }
    ///
    /// Tries to checkout signature of PC-DOS 
    /// x86 real-mode executable
    /// 
    /// @return: Optional value with Some(unit) or an io::Error prepared instance
    ///  
    pub fn validate(&self) -> io::Result<()> {
        return match self.e_magic {
            E_CIGAM => Ok(()),
            E_MAGIC => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "missing realmode executable header"
            ))
        };
    }
    ///
    /// Tries to validate checksum set in the MZ header
    /// 
    /// @returns: Some(unit) or prepared io::Error instance
    /// 
    pub fn validate_crc(buffer: &[u8]) -> io::Result<()> {
        let mut pos: usize = 0;
        let mut sum: u16 = 0;
        
        while pos < buffer.len() {
            // iterate each buffer element
            let word: [u8; 2] = [buffer[pos], *buffer.get(pos + 1).unwrap_or(&0)];
            let word: u16 = u16::from_le_bytes(word);
            sum = sum.wrapping_add(word);
            pos += 2;
        }

        return match sum {
            0 => Ok(()),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected crc value"))
        };
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
        return self.e_lfarlc == E_LFARLC;
    }
}
