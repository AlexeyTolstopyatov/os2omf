use crate::exe386;
use bytemuck::{Pod, Zeroable};
use std::io;
use std::io::{Error, Read, Seek, SeekFrom};
#[derive(Debug)]
pub struct ObjectPagesTable {
    pub pages: Vec<ObjectPage>
}
#[derive(Debug)]
pub enum ObjectPage {
    LEPageFormat(LEObjectPageHeader),
    LXPageFormat(LXObjectPageHeader),
}

#[repr(C)]
#[derive(Debug,Clone, Copy, Pod, Zeroable)]
pub struct LEObjectPageHeader {
    pub page_number: [u8; 3], // 24-bit page <-- \
    pub flags: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct LXObjectPageHeader {
    pub page_offset: u32,
    pub data_size: u16,
    pub flags: u16,
}
#[derive(Debug, Clone)]
pub struct LXObjectPageData {
    pub data: Vec<u8>,
    pub flags: PageFlags,
    pub number: u32
}
impl ObjectPagesTable {
    pub fn read<T: Read>(
        reader: &mut T,
        pages_count: u32,
        pages_shift: u32,
        magic: u16,
        ) -> io::Result<Self> {
        let mut pages = Vec::<ObjectPage>::with_capacity(pages_count as usize);

        match magic {
            exe386::header::LX_CIGAM => Self::fill_lx_pages(reader, &mut pages, pages_shift),
            exe386::header::LX_MAGIC => Self::fill_lx_pages(reader, &mut pages, pages_shift),
            exe386::header::LE_CIGAM => {},
            exe386::header::LE_MAGIC => {},
            _ => unreachable!()
        }

        Ok(Self {
            pages
        })
    }
    pub fn fill_lx_pages<T: Read>(reader: &mut T, pages: &mut Vec<ObjectPage>, pages_count: u32) {
        for _ in 0..pages_count {
            let entry = LXObjectPageHeader::read(reader).unwrap();
            pages.push(ObjectPage::LXPageFormat(entry));
        }
    }
    pub fn fill_le_pages<T: Read>(reader: &mut T, pages: &mut Vec<ObjectPage>, pages_count: u32) {
        for _ in 0..pages_count {
            let entry: LEObjectPageHeader = LEObjectPageHeader::read(reader).unwrap();
            pages.push(ObjectPage::LEPageFormat(entry));
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct PageFlags {
    pub is_legal_physical: bool,
    pub is_iterated: bool,
    pub is_invalid: bool,
    pub is_zero_filled: bool,
}
impl From<u16> for PageFlags {
    fn from(flags: u16) -> Self {
        Self {
            is_zero_filled: (flags & 0x03) != 0,
            is_invalid: (flags & 0x02) != 0,
            is_iterated: (flags & 0x01) != 0,
            is_legal_physical: (flags & 0x00) == 0 && flags != 0,
        }
    }
}
impl LEObjectPageHeader {
    pub fn read<T: Read>(reader: &mut T) -> Result<Self, Error> {
        let mut buffer = [0_u8; 4];
        reader.read_exact(&mut buffer)?;

        Ok(bytemuck::pod_read_unaligned(&buffer))
    }
}

impl LXObjectPageHeader {
    pub fn read<T: Read>(reader: &mut T) -> Result<Self, Error> {
        let mut buffer = [0_u8; 8];
        reader.read_exact(&mut buffer)?;

        Ok(bytemuck::pod_read_unaligned(&buffer))
    }
    pub fn read_page_data<R: Read + Seek>(
        reader: &mut R,
        page_entry: &LXObjectPageHeader,
        page_shift: u32,
        data_pages_offset: u64,
    ) -> io::Result<LXObjectPageData> {
        let flags = PageFlags::from(page_entry.flags);

        if flags.is_zero_filled || flags.is_invalid {
            return Ok(LXObjectPageData {
                data: vec![0; page_entry.data_size as usize],
                flags,
                number: 0, // <-- set-up it later
            });
        }

        // find real offset using page_shift
        let actual_offset = data_pages_offset + ((page_entry.page_offset as u64) << page_shift);
        reader.seek(SeekFrom::Start(actual_offset))?;

        let mut data = vec![0_u8; page_entry.data_size as usize];
        reader.read_exact(&mut data)?;

        Ok(LXObjectPageData {
            data,
            flags,
            number: 0,
        })
    }
}