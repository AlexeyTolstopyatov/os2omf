use std::io::{self, Read, Seek, SeekFrom};
///
/// This table contains one 8-byte record for every code and data segment
/// in the program or library module. 
/// 
/// Each segment has an #ordinal number
/// associated with it. For example, the first segment has an ordinal
/// number of 1. 
/// These segment numbers are used to reference the segments
/// in other sections of the New Executable file. 
/// (Offsets are from the beginning of the record.)
/// 
#[derive(Debug, Clone)]
pub struct NeSegment {
    pub header: NeSegmentHeader,
    pub shift_count: u16,
    pub data: Option<Vec<u8>>,
}

impl NeSegment {
    /// Reads the record in segments table
    /// without raw segment data.
    pub fn read<R: Read>(r: &mut R, shift_count: u16) -> io::Result<Self> {
        return Ok(Self {
            header: NeSegmentHeader::read(r)?,
            shift_count,
            data: None,
        });
    }
    /// Reads the segment data uses header information.
    pub fn read_data<R: Read + Seek>(&mut self, r: &mut R) -> io::Result<()> {
        if self.header.data_offset_shifted == 0 {
            return Ok(());
        }
        let data_offset = self.data_offset();
        let data_length = self.data_length();
        r.seek(SeekFrom::Start(data_offset))?;
        let mut data = vec![0; data_length as usize];
        r.read_exact(&mut data)?;
        self.data = Some(data);
        
        return Ok(());
    }
    /// Computes real data offset 
    pub fn data_offset(&self) -> u64 {
        return (self.header.data_offset_shifted as u64) << self.shift_count;
    }

    pub fn data_length(&self) -> u64 {
        if self.header.data_length == 0 {
            return 0x10000;
        } else {
            return self.header.data_length as u64;
        }
    }

    pub fn min_alloc(&self) -> u64 {
        if self.header.min_alloc == 0 {
            return 0x10000;
        } else {
            return self.header.min_alloc as u64;
        }
    }
}
///
/// NE Segment header is a record in Segments table
/// Like a PE32/+ files, NE executable images has a table of something which
/// contains raw code or data.                 |
///                                            |
/// +--------+--------+-------+----------+ <---+ e_lfanew + e_segtab
/// | offset | length | flags | minalloc |
/// | 0xABCD | 0x0100 | 0x... | ...      |
/// | 0xBOOC | 0x0020 | 0x007 | ...      |
/// | ...    | ...    | ...   | ...      | 
///      |                 |
///      |                 |
///      |                 +-----> Based on flags and SEG_HASMASK (0x0007) byte
///  Segments with offset = 0      defines the rules for each segment in table.
///  are .BSS prototypes           flags & HASMASK = 1 -> .CODE16 segment
///  because there's no iterated                     0 -> .DATA16 segment
///  or compressed segments       (flags & PRELOAD) + (flags & HASMASK)
///                                                 0 -> .DATA16  (read-write)
///                                                 1 -> .RDATA16 (read-only)
/// Every segment has a rights to contain own relocations table,
/// because this way to imagine the segments table is the most simple.
/// 
#[derive(Debug, Clone, Copy)]
pub struct NeSegmentHeader {
    pub data_offset_shifted: u16,
    pub data_length: u16,
    pub flags: u16,
    pub min_alloc: u16,
}
pub enum NeSegmentRights {
    /// Rights of 16-bit .code segment
    ///     - READABLE
    ///     - EXECUTABLE
    CODE = 0,
    /// Rights of 16-bit .data segment
    ///     - READABLE
    ///     - WRITABLE
    DATA = 1,
    /// Rights of 16-bit .rodata segment
    ///     - READABLE
    RDATA= 2,
    /// My custom new-type of segment without
    /// embedded data. In original documents there's no
    /// any .bss named segments and all segments are unnamed by nature,
    /// Rights of 16-bit ~.bss~ data segment defines by the flags
    BSS = 3
}
const SEG_HASMASK: u16 = 0x0007;
const SEG_MOVABLE: u16 = 0x0010;
const SEG_PRELOAD: u16 = 0x0040;
const SEG_RELOCS:  u16 = 0x0100;
const SEG_DISCARD: u16 = 0xF000;

impl NeSegmentHeader {
    pub fn read<TR: Read>(r: &mut TR) -> io::Result<Self> {
        let mut buf = [0; 0x8];
        r.read_exact(&mut buf)?;
        let get_u16 = |pos| u16::from_le_bytes(buf[pos..pos + 2].try_into().unwrap());

        Ok(Self {
            data_offset_shifted: get_u16(0),
            data_length: get_u16(2),
            flags: get_u16(4),
            min_alloc: get_u16(6),
        })
    }
    pub fn get_segment_rights(&self) -> NeSegmentRights {
        if self.data_offset_shifted == 0_u16 {
            return NeSegmentRights::BSS;
        }

        return match (self.flags & SEG_HASMASK) == 0 {
            true => NeSegmentRights::CODE,
            false => {
                if (self.flags & SEG_PRELOAD) != 0 {
                    return NeSegmentRights::RDATA;
                } else {
                    return NeSegmentRights::DATA;
                }
            }
        }
    }
    pub fn get_segment_flags_list() {
        
    }
}
