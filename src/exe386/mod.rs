use crate::exe::{MzHeader, E_CIGAM, E_MAGIC};
use crate::exe386::dirtab::ModuleDirectivesTable;
use crate::exe386::enttab::EntryTable;
use crate::exe386::fpagetab::FixupPageTable;
use crate::exe386::frectab::FixupRecordsTable;
use crate::exe386::header::{LinearExecutableHeader, LX_CIGAM, LX_MAGIC};
use crate::exe386::imptab::{ImportData, ImportRelocationsTable};
use crate::exe386::objpagetab::ObjectPagesTable;
use crate::exe386::objtab::ObjectsTable;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Read, Seek, SeekFrom};

pub mod header;
pub mod vxd;
pub mod objtab;
pub mod enttab;
pub mod frectab;
pub mod imptab;
pub mod nrestab;
pub mod resntab;
pub mod objpagetab;
pub mod fpagetab;
pub mod dirtab;

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
    pub fn get(path: &str) -> Result<Self, Error> {
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

        let offset = |ptr: u32| -> u64 {
            ptr as u64 + base_offset
        };

        if !header.has_valid_magic() { 
            return Err(Error::new(ErrorKind::Other, "Unable to read module as linear executable"));
        }
        
        let object_pages = ObjectPagesTable::read(
            &mut reader,
            offset(header.e32_objmap),
            header.e32_mpages,
            header.e32_pageshift_or_lastpage,
            header.e32_magic
        )?;
        let object_table = ObjectsTable::read(
            &mut reader,
            offset(header.e32_objtab),
            header.e32_objcnt
        )?;
        reader.seek(SeekFrom::Start(offset(header.e32_enttab)))?;
        let entry_table = EntryTable::read(
            &mut reader,
            offset(header.e32_enttab)
        )?;
        reader.seek(SeekFrom::Start(offset(header.e32_fpagetab)))?;
        let fixup_page_table = FixupPageTable::read(
            &mut reader, 
            &header
        )?;
        let fixup_records = FixupRecordsTable::read(
            &mut reader,
            &fixup_page_table,
            offset(header.e32_frectab)
        )?;
        let import_table = ImportRelocationsTable::read(
            &mut reader,
            ImportData{
                imp_mod_offset: offset(header.e32_impmod),
                imp_proc_offset: offset(header.e32_impproc),
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
            fixup_page_table,
            module_directives_table
        })
    }
}