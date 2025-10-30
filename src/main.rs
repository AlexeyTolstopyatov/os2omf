use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Seek, SeekFrom};
use crate::exe286::header::NeHeader;
use crate::exe286::NE_MAGIC;

mod exe;
mod exe286;
mod exe386;

struct NeExecutableStruct {
    dos_header: Box<exe::MzHeader>,
    win_header: Box<exe286::header::NeHeader>,
    seg_tab: Vec<exe286::segtab::NeSegment>,
    seg_rel: Vec<exe286::segrelocs::RelocationTable>,
    ent_tab: Box<exe286::enttab::EntryTable>,
    nres_tab: Box<exe286::nrestab::NonResidentNameTable>,
    resn_tab: Box<exe286::resntab::ResidentNameTable>,
}
pub fn fill_header<TRead: Read + Seek>(reader: &mut TRead) -> Result<(), io::Error> {
    let dos_header = exe::MzHeader::read(reader)?;
    reader.seek(SeekFrom::Start(dos_header.e_lfanew as u64))?;

    let win_header = exe286::header::NeHeader::read(reader)?;

    Ok(())
}
/// It will be Dynamic linked object later
///  - rustc 1.88.0 (6b00bc388 2025-06-23)
///  - bytemuck 1.24.0
fn main() {
    let path = "D:\\TEST\\Windows2.1\\CALC.EXE";
    let mut reader = BufReader::new(File::open(path).unwrap());

    let dos_header = exe::MzHeader::read(&mut reader).unwrap(); // dangerous!
    if !dos_header.has_valid_magic() {
        return;
    }
    dbg!(dos_header);

    reader.seek(SeekFrom::Start(dos_header.e_lfanew as u64)).unwrap();

    let win_header = exe286::header::NeHeader::read(&mut reader).unwrap();

    if !win_header.is_valid_magic() {
        return;
    }
    dbg!(win_header);
}
