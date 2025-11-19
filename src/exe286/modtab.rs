use std::io::{self, Read};
///
/// Module References Table
/// Represents WORD records array where count of records defines
/// in the IMAGE_OS2_HEADER or a NE Header -- `e_cmod`
/// ```
///                                 |
///     e_modtab is relative        | e_lfanew + e_modtab = file offset
///     offset from NE header       |
/// +---------------+ <-------------+
/// | 0x0001        | <-- may be "MSG" = |03|__|__|__| (skip 4 bytes)
/// | 0x0004        |     after "MSG" Pascal-String follows "KERNEL"
/// | ...           |     Pascal-string (in example)
/// ```
/// Module references is an offsets from ImportNames Table start.
/// To get the "KERNEL" module string you need to get an offset from e_modtab
/// 1) Select a 2nd offset (modtab[[1]] = 0x0004)
/// 2) e_lfanew + e_imptab + modtab[[1]]
/// 3) Read the Pascal-String
pub struct ModuleReferencesTable {
    pub m_offsets: Vec<u16>,
}

impl ModuleReferencesTable {
    pub fn read<TRead: Read>(reader: &mut TRead, e_modtab: u64, cmod: u16) -> io::Result<Self> {
        let mut references: Vec<u16> = Vec::<u16>::new();
        let mut buf: [u8; 2] = [0, 0];

        for _ in 0..cmod {
            reader.read_exact(&mut buf)?;
            references.push(bytemuck::cast(buf));
        }

        Ok(ModuleReferencesTable {
            m_offsets: references,
        })
    }
}
