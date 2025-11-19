use crate::exe::MzHeader;
use crate::exe386::dirtab::ModuleDirectivesTable;
use crate::exe386::enttab::EntryTable;
use crate::exe386::fpagetab::FixupPageTable;
use crate::exe386::frectab::FixupRecordsTable;
use crate::exe386::header::LinearExecutableHeader;
use crate::exe386::imptab::{ImportData, ImportRelocationsTable};
use crate::exe386::objpagetab::ObjectPagesTable;
use crate::exe386::objtab::ObjectsTable;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Read, Seek, SeekFrom};

pub mod dirtab;
pub mod enttab;
pub mod fpagetab;
pub mod frectab;
pub mod header;
pub mod imptab;
pub mod nrestab;
pub mod objpagetab;
pub mod objtab;
pub mod resntab;
pub mod vxd;

pub(crate) struct LinearExecutableLayout {
    pub header: LinearExecutableHeader,
    pub object_table: ObjectsTable,
    pub object_pages: ObjectPagesTable,
    pub entry_table: EntryTable,
    pub fixup_page_table: FixupPageTable,
    //pub fixup_records_table: FixupRecordsTable,
    pub import_table: ImportRelocationsTable,
    pub module_directives_table: ModuleDirectivesTable,
}

impl LinearExecutableLayout {
    ///
    /// Linear executables unlike other legacy formats
    /// may not contain DOS compatibility (MZ header missing)
    /// Then first header instead of DOS header will be Linear Executable header
    /// and all relative pointers what set in header becomes absolute
    ///
    /// Returns
    ///
    fn define_base_offset<T: Read>(reader: &mut T) -> Option<u64> {
        let maybe_header = MzHeader::read(reader);
        match maybe_header {
            Ok(h) => {
                return Some(h.e_lfanew as u64)
            },
            Err(..) => {
                // ignore for 1st time
            }
        }

        let maybe_header = LinearExecutableHeader::read(reader);
        match maybe_header {
            Ok(_) => Some(0),
            Err(..) => None,
        }
    }

    pub fn get(path: &str) -> Result<Self, Error> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let base_offset = match Self::define_base_offset(&mut reader) {
            Some(offset) => offset,
            None => Err(Error::new(ErrorKind::InvalidInput, "Could not determine base offset"))?,
        };
        reader.seek(SeekFrom::Start(base_offset))?;
        let header = LinearExecutableHeader::read(&mut reader)?;

        let offset = |ptr: u32| -> u64 { ptr as u64 + base_offset };

        let object_pages = ObjectPagesTable::read(
            &mut reader,
            offset(header.e32_objmap),
            header.e32_mpages,
            header.e32_pageshift_or_lastpage,
            header.e32_magic,
        )?;
        let object_table = ObjectsTable::read(
            &mut reader,
            offset(header.e32_objtab),
            header.e32_objcnt
        )?;
        let entry_table = EntryTable::read(
            &mut reader,
            offset(header.e32_enttab)
        )?;
        let fixup_page_table = FixupPageTable::read(
            &mut reader,
            offset(header.e32_fpagetab),
            &header
        )?;
        let fixup_records = FixupRecordsTable::read(
            &mut reader,
            &fixup_page_table,
            offset(header.e32_frectab)
        )?;
        let import_table = ImportRelocationsTable::read(
            &mut reader,
            ImportData {
                imp_mod_offset: offset(header.e32_impmod),
                imp_proc_offset: offset(header.e32_impproc),
                fixup_records: fixup_records.records,
            },
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
            module_directives_table,
        })
    }
}
