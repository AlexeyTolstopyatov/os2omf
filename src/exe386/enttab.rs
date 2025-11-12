use std::io;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct EntryBundle {
    pub count: u8,
    pub bundle_type: BundleType,
    pub object: u16,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BundleType {
    Unused,
    Entry16,
    Entry286CallGate,
    Entry32,
    Forwarder,
    Unknown(u8),
}
impl From<u8> for BundleType {
    fn from(value: u8) -> Self {
        match value & 0x7F {
            0x00 => BundleType::Unused,
            0x01 => BundleType::Entry16,
            0x02 => BundleType::Entry286CallGate,
            0x03 => BundleType::Entry32,
            0x04 => BundleType::Forwarder,
            n => BundleType::Unknown(n),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EntryTable {
    pub bundles: Vec<EntryBundle>,
}

#[derive(Debug, Clone, Copy)]
pub struct Entry16 {
    pub flags: u8,
    pub offset: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct Entry32 {
    pub flags: u8,
    pub offset: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct EntryCallGate {
    pub flags: u8,
    pub offset: u16,
    pub callgate_selector: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct EntryForwarder {
    pub flags: u8,
    pub module_ordinal: u16,
    pub offset_or_ordinal: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum Entry {
    Unused,
    Entry16(Entry16),
    Entry32(Entry32),
    EntryCallGate(EntryCallGate),
    EntryForwarder(EntryForwarder),
}

impl EntryTable {
    pub fn read<T: Read>(reader: &mut T) -> io::Result<Self> {
        let mut bundles = Vec::new();

        loop {
            let mut count_buf = [0u8];
            reader.read_exact(&mut count_buf)?;
            let count = count_buf[0];

            if count == 0 {
                break;
            }

            let mut type_buf = [0u8];
            reader.read_exact(&mut type_buf)?;
            let bundle_type = BundleType::from(type_buf[0]);

            let object = if bundle_type != BundleType::Unused && bundle_type != BundleType::Forwarder {
                let mut obj_buf = [0u8; 2];
                reader.read_exact(&mut obj_buf)?;
                u16::from_le_bytes(obj_buf)
            } else {
                0
            };

            let mut entries = Vec::with_capacity(count as usize);
            for _ in 0..count {
                let entry = match bundle_type {
                    BundleType::Unused => Entry::Unused,
                    BundleType::Entry16 => {
                        let entry_data = Entry16::read(reader)?;
                        Entry::Entry16(entry_data)
                    },
                    BundleType::Entry286CallGate => {
                        let entry_data = EntryCallGate::read(reader)?;
                        Entry::EntryCallGate(entry_data)
                    },
                    BundleType::Entry32 => {
                        let entry_data = Entry32::read(reader)?;
                        Entry::Entry32(entry_data)
                    },
                    BundleType::Forwarder => {
                        let entry_data = EntryForwarder::read(reader)?;
                        Entry::EntryForwarder(entry_data)
                    },
                    BundleType::Unknown(unknown_type) => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Unknown bundle type: 0x{:02x}", unknown_type)
                        ));
                    }
                };
                entries.push(entry);
            }

            bundles.push(EntryBundle {
                count,
                bundle_type,
                object,
                entries,
            });
        }

        Ok(EntryTable { bundles })
    }
}

impl Entry16 {
    pub fn read<T: Read>(reader: &mut T) -> io::Result<Self> {
        let mut flags_buf = [0_u8];
        reader.read_exact(&mut flags_buf)?;

        let mut offset_buf = [0_u8; 2];
        reader.read_exact(&mut offset_buf)?;

        Ok(Entry16 {
            flags: flags_buf[0],
            offset: u16::from_le_bytes(offset_buf),
        })
    }
}

impl Entry32 {
    pub fn read<T: Read>(reader: &mut T) -> io::Result<Self> {
        let mut flags_buf = [0_u8];
        reader.read_exact(&mut flags_buf)?;

        let mut offset_buf = [0_u8; 4];
        reader.read_exact(&mut offset_buf)?;

        Ok(Entry32 {
            flags: flags_buf[0],
            offset: u32::from_le_bytes(offset_buf),
        })
    }
}

impl EntryCallGate {
    pub fn read<T: Read>(reader: &mut T) -> io::Result<Self> {
        let mut flags_buf = [0_u8];
        reader.read_exact(&mut flags_buf)?;

        let mut offset_buf = [0_u8; 2];
        reader.read_exact(&mut offset_buf)?;

        let mut callgate_buf = [0_u8; 2];
        reader.read_exact(&mut callgate_buf)?;

        Ok(EntryCallGate {
            flags: flags_buf[0],
            offset: u16::from_le_bytes(offset_buf),
            callgate_selector: u16::from_le_bytes(callgate_buf),
        })
    }
}

impl EntryForwarder {
    pub fn read<T: Read>(reader: &mut T) -> io::Result<Self> {
        let mut reserved_buf = [0u8; 2];
        reader.read_exact(&mut reserved_buf)?;

        let mut flags_buf = [0_u8];
        reader.read_exact(&mut flags_buf)?;

        let mut module_ordinal_buf = [0_u8; 2];
        reader.read_exact(&mut module_ordinal_buf)?;

        let mut offset_or_ordinal_buf = [0_u8; 4];
        reader.read_exact(&mut offset_or_ordinal_buf)?;

        Ok(EntryForwarder {
            flags: flags_buf[0],
            module_ordinal: u16::from_le_bytes(module_ordinal_buf),
            offset_or_ordinal: u32::from_le_bytes(offset_or_ordinal_buf),
        })
    }
}