use crate::exe::{MzHeader, E_CIGAM, E_MAGIC};
use crate::exe386::enttab::EntryTable;
use crate::exe386::frectab::FixupRecordsTable;
use crate::exe386::header::{LinearExecutableHeader, LX_CIGAM, LX_MAGIC};
use crate::exe386::imptab::{ImportData, ImportRelocationsTable};
use crate::exe386::objtab::ObjectsTable;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Read, Seek, SeekFrom};
use crate::exe386::dirtab::ModuleDirectivesTable;
use crate::exe386::fpagetab::FixupPageTable;
use crate::exe386::objpagetab::{ObjectPage, ObjectPagesTable};

pub mod header;
pub mod vxd;
pub mod objtab;
pub mod enttab;
pub mod frectab;
pub mod imptab;
mod nrestab;
mod resntab;
mod objpagetab;
mod fpagetab;
mod dirtab;

pub(crate) struct LinearExecutableLayout {
    pub header: LinearExecutableHeader,
    pub object_table: ObjectsTable,
    pub object_pages: ObjectPagesTable,
    pub entry_table: EntryTable,
    pub fixup_page_table: FixupPageTable,
    //pub fixup_records_table: FixupRecordsTable,
    pub import_table: ImportRelocationsTable,
    pub module_directives_table: ModuleDirectivesTable
}

impl LinearExecutableLayout {
    pub fn read(path: &str) -> Result<Self, Error> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut sig_buffer = [0_u8; 2];
        reader.read_exact(&mut sig_buffer)?;

        let sig_bytes = u16::from_be_bytes(sig_buffer);
        let mut base_offset: u64 = 0;
        reader.seek(SeekFrom::Start(0))?;

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
        reader.seek(SeekFrom::Start(__offset(header.e32_objmap)))?;
        let object_pages = ObjectPagesTable::read(&mut reader, header.e32_mpages, header.e32_pageshift, header.e32_magic)?;
        
        reader.seek(SeekFrom::Start(__offset(header.e32_objtab)))?;
        let object_table = ObjectsTable::read(&mut reader, header.e32_objcnt)?;
        
        reader.seek(SeekFrom::Start(__offset(header.e32_enttab)))?;
        let entry_table = EntryTable::read(&mut reader)?;

        reader.seek(SeekFrom::Start(__offset(header.e32_fpagetab)))?;
        let fixup_pages = FixupPageTable::read(&mut reader, &header)?;

        reader.seek(SeekFrom::Start(__offset(header.e32_frectab)))?;
        let fixup_records = FixupRecordsTable::read(
            &mut reader,
            &fixup_pages,
            __offset(header.e32_frectab) // <-- expected relative or an absolute value?
        )?;

        let import_table = ImportRelocationsTable::read(
            &mut reader,
            ImportData{
                imp_mod_offset: __offset(header.e32_impmod),
                imp_proc_offset: __offset(header.e32_impproc),
                fixup_records: fixup_records.records,
            }
        )?;

        let mut module_directives_table = ModuleDirectivesTable::empty();

        if header.e32_dirtab != 0 {
            module_directives_table = ModuleDirectivesTable::read(
                &mut reader,
                &header,
                base_offset
            )?;
        }

        Ok(Self {
            header,
            object_table,
            object_pages,
            entry_table,
            import_table,
            fixup_page_table: fixup_pages,
            module_directives_table
        })
    }
}