use std::io::{Error, Read, Seek};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Zeroable, Pod)]
pub struct LXPage {
    pub page_offset: u32,
    pub page_data_size: u16,
    pub page_flags: u16,
}
pub enum LXPageFlag {
//  00h = Legal Physical Page in the module (Offset from Preload Page Section).
//  01h = Iterated Data Page (Offset from Iterated Data Pages Section).
//  02h = Invalid Page (zero).
//  03h = Zero Filled Page (zero).
//  04h = Unused.
//  05h = Compressed Page (Offset from Preload Pages Section)
    Legal = 0x00000,
    Iterated = 0x00001,
    Invalid = 0x00002,
    Zeroed = 0x00003,
    Unused = 0x00004,
    Compressed = 0x00005,
}

pub struct LXObjectPageTable {
    pages: Vec<LXPage>,
}
impl LXObjectPageTable {
    pub fn new<T: Read + Seek>(r: &mut T, pages: u32) -> Result<LXObjectPageTable, Error> {
        let mut vec = Vec::<LXPage>::new();

        for _ in 0..pages {
            let mut caught_page = [0_u8; 8];
            r.read_exact(&mut caught_page)?;
            vec.push(bytemuck::cast(caught_page));
        }

        Ok(LXObjectPageTable{
            pages: vec,
        })
    }
}