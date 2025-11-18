use std::convert::TryInto;
use std::io::{self, Read, Seek, SeekFrom};


///
/// This table contains one member for every entry point in the program (EXE/DRV/SYS) or
/// library module (DLL).
/// (Every public FAR function or procedure in a module is
/// an entry point.)
///
/// The members in the entry table have ordinal numbers
/// beginning at 1.
/// These ordinal numbers are referenced by the resident
/// names table and the nonresident names table.
///
/// LINK versions 4.0 and later bundle the members of the entry table.
/// Each bundle begins with the following information. (Offsets are from
/// the beginning of the bundle.)
///
/// Open Watcom 1.8 links NE segmented programs correctly (bases on Microsoft link 5.10)
///
#[derive(Debug, Clone)]
pub struct EntryTable {
    pub entries: Vec<Entry>,
}

impl EntryTable {
    pub fn read<R: Read + Seek>(reader: &mut R, e_enttab: u64, cb_ent_tab: u16) -> io::Result<Self> {
        let mut entries: Vec<Entry> = Vec::new();
        // In practice: pointer checking optional operation too
        // If file really linked as New Executable (by Microsoft LINK.EXE)
        // Independent on format version -- wrong pointer *always* return empty entry table
        reader.seek(SeekFrom::Start(e_enttab))?;
        let mut bytes_remaining = cb_ent_tab;
        let mut _ordinal: u16 = 1; // entry index means ordinal in non/resident names tables

        while bytes_remaining > 0 {
            // Read bundle header
            let mut buffer = [0; 2];
            reader.read_exact(&mut buffer)?;
            bytes_remaining -= 2;

            let entries_count = buffer[0];
            let seg_id = buffer[1];

            if entries_count == 0 {
                // End of table marker
                break;
            }

            if seg_id == 0 {
                // Unused entries (padding between actual entries)
                for _ in 0..entries_count {
                    entries.push(Entry::Unused);
                    _ordinal += 1;
                }
                continue;
            }

            // Calculate bundle size based on segment type
            let entry_size = match seg_id == 0xFF {
                true => 6,
                false => 3,
            };
            let bundle_size = (entries_count as u16) * entry_size;

            if bundle_size > bytes_remaining {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Bundle size exceeds remaining bytes: bundle_size={}, remaining={}",
                            bundle_size, bytes_remaining),
                ));
            }
            bytes_remaining -= bundle_size;

            for _ in 0..entries_count {
                let entry = if seg_id == 0xFF {
                    Entry::Moveable(MoveableEntry::read(reader)?)
                } else {
                    Entry::Fixed(FixedEntry::read(reader, seg_id)?)
                };
                entries.push(entry);
                _ordinal += 1;
            }
        }

        Ok(Self { entries })
    }
}

#[derive(Debug, Clone)]
pub enum Entry {
    Unused,
    Fixed(FixedEntry),
    Moveable(MoveableEntry),
}

#[derive(Debug, Clone, Copy)]
pub struct FixedEntry {
    pub segment: u8,
    pub flags: u8,
    pub offset: u16,
}

impl FixedEntry {
    pub fn read<R: Read>(r: &mut R, segment: u8) -> io::Result<Self> {
        let mut buf = [0; 3];
        r.read_exact(&mut buf)?;
        Ok(Self {
            segment,
            flags: buf[0],
            offset: u16::from_le_bytes(buf[1..3].try_into().unwrap()),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MoveableEntry {
    pub flags: u8,
    pub magic: [u8; 2],
    pub segment: u8,
    pub offset: u16,
}

impl MoveableEntry {
    pub fn read<TRead: Read>(r: &mut TRead) -> io::Result<Self> {
        let mut buf = [0; 6];
        r.read_exact(&mut buf)?;
        Ok(Self {
            flags: buf[0],
            magic: [buf[1], buf[2]],
            segment: buf[3],
            offset: u16::from_le_bytes(buf[4..6].try_into().unwrap()),
        })
    }
}