use crate::exe386::frectab::{FixupRecord, FixupTarget};
use crate::types::PascalString;
use std::io::{self, Error, ErrorKind, Read, Seek, SeekFrom};

#[derive(Debug)]
pub enum ImportError {
    Io(io::Error),
    InvalidModuleOrdinal(u16),
    InvalidStringLength(u8),
}

#[derive(Debug, Clone)]
pub struct ImportData {
    pub imp_mod_offset: u64,
    pub imp_proc_offset: u64,
    pub fixup_records: Vec<FixupRecord>,
}

#[derive(Debug, Clone)]
pub struct ImportRelocationsTable {
    imports: Vec<DllImport>,
}

impl ImportRelocationsTable {
    pub fn imports(&self) -> &[DllImport] {
        &self.imports
    }

    fn read_modules<T: Read + Seek>(
        reader: &mut T,
        imp_mod_offset: u64,
    ) -> io::Result<Vec<PascalString>> {
        if imp_mod_offset == 0 {
            return Ok(Vec::new());
        }

        let original_pos = reader.stream_position()?;
        reader.seek(SeekFrom::Start(imp_mod_offset))?;

        let mut modules = Vec::new();

        loop {
            let len = Self::read_byte(reader)?;
            if len == 0 {
                break;
            }

            let name_bytes = Self::read_bytes(reader, len as usize)?;
            modules.push(PascalString::new(len, name_bytes));
        }

        reader.seek(SeekFrom::Start(original_pos))?;
        Ok(modules)
    }

    fn read_pascal_string<T: Read>(reader: &mut T) -> io::Result<PascalString> {
        let len = Self::read_byte(reader)?;
        if len == 0 {
            return Ok(PascalString::empty());
        }

        let name_bytes = Self::read_bytes(reader, len as usize)?;
        Ok(PascalString::new(len, name_bytes))
    }

    fn read_byte<T: Read>(reader: &mut T) -> io::Result<u8> {
        let mut buf = [0u8];
        reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_bytes<T: Read>(reader: &mut T, count: usize) -> io::Result<Vec<u8>> {
        let mut buf = vec![0u8; count];
        reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn process_imported_name<T: Read + Seek>(
        reader: &mut T,
        name_target: &crate::exe386::frectab::FixupTargetImportedName,
        modules: &[PascalString],
        imp_proc_offset: u64,
    ) -> Result<DllImport, Error> {
        let module_index = name_target.module_ordinal - 1;
        let module_name = modules
            .get(module_index as usize)
            .ok_or_else(|| ImportError::InvalidModuleOrdinal(module_index))
            .unwrap()
            .clone();

        let procedure_ptr = imp_proc_offset + name_target.procedure_name_offset as u64;

        let original_pos = reader.stream_position()?;
        reader.seek(SeekFrom::Start(procedure_ptr))?;

        let import_name = Self::read_pascal_string(reader)?;

        reader.seek(SeekFrom::Start(original_pos))?;

        Ok(DllImport::ImportName(DllImportName {
            module_index,
            module_name,
            import_name_offset: name_target.procedure_name_offset,
            import_name,
        }))
    }

    fn process_imported_ordinal(
        ordinal_target: &crate::exe386::frectab::FixupTargetImportedOrdinal,
        modules: &[PascalString],
    ) -> Result<DllImport, Error> {
        let module_index = ordinal_target.module_ordinal -1;
        let module_name = modules
            .get(module_index as usize)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("Module at {} index is invalid", module_index)))?
            .clone();

        Ok(DllImport::ImportOrdinal(DllImportOrdinal {
            module_index,
            module_name,
            import_ordinal: ordinal_target.import_ordinal,
        }))
    }

    pub fn read<T: Read + Seek>(reader: &mut T, import_data: ImportData) -> Result<Self, Error> {
        let modules = Self::read_modules(reader, import_data.imp_mod_offset)?;
        let mut imports = Vec::new();

        for record in import_data.fixup_records {
            let is_import_reloc = matches!(
                record.target_data,
                FixupTarget::ImportedName(_) | FixupTarget::ImportedOrdinal(_)
            );

            if !is_import_reloc {
                continue;
            }

            match record.target_data {
                FixupTarget::ImportedName(ref name_target) => {
                    let import = Self::process_imported_name(
                        reader,
                        name_target,
                        &modules,
                        import_data.imp_proc_offset,
                    )?;
                    imports.push(import);
                }
                FixupTarget::ImportedOrdinal(ref ordinal_target) => {
                    let import = Self::process_imported_ordinal(ordinal_target, &modules)?;
                    imports.push(import);
                }
                _ => unreachable!(),
            }
        }

        Ok(Self { imports })
    }
}

#[derive(Debug, Clone)]
pub enum DllImport {
    ImportName(DllImportName),
    ImportOrdinal(DllImportOrdinal),
}

impl DllImport {
    pub fn module_name(&self) -> &PascalString {
        match self {
            DllImport::ImportName(import) => &import.module_name,
            DllImport::ImportOrdinal(import) => &import.module_name,
        }
    }

    pub fn module_index(&self) -> u16 {
        match self {
            DllImport::ImportName(import) => import.module_index,
            DllImport::ImportOrdinal(import) => import.module_index,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DllImportName {
    pub module_index: u16,
    pub module_name: PascalString,
    pub import_name_offset: u32,
    pub import_name: PascalString,
}

#[derive(Debug, Clone)]
pub struct DllImportOrdinal {
    pub module_index: u16,
    pub module_name: PascalString,
    pub import_ordinal: u32,
}