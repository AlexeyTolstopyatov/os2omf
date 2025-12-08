use crate::exe::MzHeader;
use bytemuck::{Pod, Zeroable};
use std::io;
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug, Clone)]
pub struct MzRelocationTable {
    relocations: Vec<FarPointer>,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct FarPointer {
    pub segment: u16,
    pub offset: u16,
}
impl MzRelocationTable {
    pub fn read<T: Read + Seek>(reader: &mut T, header: &MzHeader) -> io::Result<Self> {
        let mut relocations = Vec::<FarPointer>::new();
        reader.seek(SeekFrom::Start(header.e_lfarlc as u64))?;

        for _ in 0..header.e_crlc {
            let mut far_buff = [0_u8; 4];
            reader.read_exact(&mut far_buff)?;
            relocations.push(bytemuck::pod_read_unaligned(&far_buff))
        }

        Ok(Self { relocations })
    }
}
