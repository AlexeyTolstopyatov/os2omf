use std::io::{Read, Seek, SeekFrom};
use crate::exe386::frectab::FixupRecordsTable;
use crate::types::PascalString;

///
/// To find the data about importing procedures
/// extremely needed to fill those fields.
///
pub struct ImportData {
    pub imp_mod_offset: u64,
    pub imp_proc_offset: u64,
    pub fixup_records_table: FixupRecordsTable,
}
impl ImportData {
    pub fn get_modules<T: Read + Seek>(&self, r: &mut T) -> Vec<PascalString> {
        let mut modules = Vec::<PascalString>::new();
        if self.imp_mod_offset == 0 {
            return modules;
        }
        let mut len = [0];
        r.seek(SeekFrom::Start(self.imp_mod_offset + 1)).unwrap();
        r.read_exact(&mut len).unwrap();

        while len[0] != 0 {
            modules.push(Self::get_pascal_string(r));
            r.read_exact(&mut len).unwrap();
        }
        modules
    }
    fn get_pascal_string<T: Read>(r: &mut T) -> PascalString {
        let len = {
            let mut len = 0;
            r.read_exact(std::slice::from_mut(&mut len)).unwrap();
            len
        };

        if len == 0 {
            return PascalString::empty();
        }

        let name = {
            let mut name = vec![0; len as usize];
            r.read_exact(&mut name).unwrap();
            name
        };

        PascalString::new(len, name)
    }
}
pub struct ImportRelocationsTable {
    imports: Vec<DllImport>
}
impl ImportRelocationsTable {
    pub fn read<T: Read + Seek>(r: &mut T, import_data: ImportData) -> Self {
        let mut imports = Vec::<DllImport>::new();
        let modules = import_data.get_modules(r);
        let just_import_relocs: Vec<_> = import_data
            .fixup_records_table
            .records
            .iter()
            .filter(|record| {(record.target_flags & 0x7F == 0x01) || (record.target_flags & 0x7F) == 0x02})
            .collect();

        for reloc in just_import_relocs {
            
        }

        Self {
            imports
        }
    }
}
pub enum DllImport {
    ImportName(DllImportName),
    ImportOrdinal(DllImportOrdinal),
}

pub struct DllImportName {
    pub module_index: u16,
    pub import_name_offset: u32,
    pub import_name: PascalString
}
pub struct DllImportOrdinal {
    pub module_index: u16,
    pub import_ordinal: u32,
}