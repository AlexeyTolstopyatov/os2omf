use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Seek, SeekFrom};
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
    if win_header.e_magic != NE_MAGIC {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid header's magic signature"));
    }

    Ok(())
}

fn main() {
    let path = "D:\\TEST\\Windows2.1\\CALC.EXE";
    let mut reader = BufReader::new(File::open(path).unwrap());

    let dos_header = exe::MzHeader::read(&mut reader).unwrap(); // dangerous!
    if !dos_header.has_valid_magic() {
        // free buffer? or it init on the stack?
        return;
    }
    dbg!(dos_header);

    let win_header = exe286::header::NeHeader::read(&mut reader).unwrap();
    dbg!(win_header);
}
