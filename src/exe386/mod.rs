use std::fmt::Debug;
use crate::exe::{MzHeader, E_CIGAM, E_MAGIC};
use crate::exe386::enttab::EntryTable;
use crate::exe386::header::{LinearExecutableHeader, LX_CIGAM, LX_MAGIC};
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Read, Seek, SeekFrom};
use crate::exe386::frectab::FixupRecordsTable;
use crate::exe386::imptab::{ImportData, ImportRelocationsTable};

pub mod header;
pub mod vxd;
pub mod objtab;
pub mod lx_objpages;
pub mod le_objpages;
pub mod enttab;
pub mod frectab;
pub mod imptab;

pub(crate) struct LinearExecutableLayout {
    pub header: LinearExecutableHeader,
    //pub objects: ObjectsTable,
    //pub object_page: LXObjectPageTable,
    pub entry_table: EntryTable,
    pub import_table: ImportRelocationsTable
}

impl LinearExecutableLayout {
    pub fn from(path: &str) -> Result<Self, Error> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut sig_buffer = [0_u8; 2];
        reader.read_exact(&mut sig_buffer)?;

        let sig_bytes = u16::from_be_bytes(sig_buffer);
        let mut base_offset: u64 = 0;
        reader.seek(SeekFrom::Start(0))?; // <-- reset position

        // firstly check new IBM's kind of binaries:
        let mut header: LinearExecutableHeader;

        if sig_bytes == LX_CIGAM || sig_bytes == LX_MAGIC {
            header = LinearExecutableHeader::read(&mut reader)?;
        }
        if sig_bytes == E_CIGAM || sig_bytes == E_MAGIC {
            let dos_header = MzHeader::read(&mut reader)?;
            base_offset = dos_header.e_lfanew as u64;
            reader.seek(SeekFrom::Start(dos_header.e_lfanew as u64))?;
        }
        else {
            return Err(Error::new(ErrorKind::Other, "Unable to read module as linear executable"));
        }

        header = LinearExecutableHeader::read(&mut reader)?;

        let __offset = |ptr: u32| -> u64 {
            ptr as u64 + base_offset
        };

        if header.e32_magic != LX_CIGAM && header.e32_magic != LX_MAGIC {
            return Err(Error::new(ErrorKind::Other, "Unable to read module as linear executable"));
        }
        reader.seek(SeekFrom::Start(__offset(header.e32_enttab)))?;
        let entry_table = EntryTable::read(&mut reader)?;

        reader.seek(SeekFrom::Start(__offset(header.e32_frectab)))?;
        let fixup_records = FixupRecordsTable::read(
            &mut reader,
            header.e32_impmod,
            header.e32_fpagetab,
            header.e32_fixupsize,
        )?;

        let imports = ImportRelocationsTable::read(
            &mut reader,
            ImportData{
                imp_mod_offset: __offset(header.e32_impmod),
                imp_proc_offset: __offset(header.e32_impproc),
                fixup_records: fixup_records.records,
            }
        )?;

        Ok(Self {
            header,
            entry_table,
            import_table: imports
        })
    }
}