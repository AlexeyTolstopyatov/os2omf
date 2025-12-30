//! This module represents structure and implementation details of `ObjectsTable`.
//! Linear executables contain segments where code or data stores. Segments like in NE format
//! may be normal (`CODE32`, `DATA32` segments) or compressed/iterated.
//! 
//! Objects are unnamed and permissions of them `LNK386.EXE` puts in characteristics.
//! Field which named `flags` stores characteristics for each object.
use bytemuck::{Pod, Zeroable};
use std::io::{Error, Read, Seek, SeekFrom};

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct Object {
    pub virtual_size: u32,
    pub virtual_addr: u32,
    pub flags: u32,
    pub map_index: u32,
    pub map_size: u32,
    pub _reserved: u32,
}
impl Object {
    pub fn get_object_rights(&self) -> LXObjectRights {
        if self.virtual_size == 0 {
            return LXObjectRights::BSS;
        }

        match self.flags & 0x0002 {
            0 => LXObjectRights::SETTER,
            3 => LXObjectRights::DATA,
            6 => LXObjectRights::CODE,
            _ => LXObjectRights::RDATA,
        }
    }
}
const OBJ_READABLE: u16 =        0x0001;
const OBJ_WRITEABLE: u16 =       0x0002;
const OBJ_EXECUTABLE: u16 =      0x0004;
const OBJ_RESOURCE: u16 =        0x0008;
const OBJ_DISCARDABLE: u16 =     0x0010;
const OBJ_SHARABLE: u16 =        0x0020;
const OBJ_HAS_PRELOAD: u16 =     0x0040;
const OBJ_HAS_INVALID: u16 =     0x0080;
const OBJ_PERM_SWAPPABLE: u16 =  0x0100;  /* LE */
const OBJ_HAS_ZERO_FILL: u16 =   0x0100;  /* LX */
const OBJ_PERM_RESIDENT: u16 =   0x0200;
const OBJ_PERM_CONTIGUOUS: u16 = 0x0300;  /* LX */
const OBJ_PERM_LOCKABLE: u16 =   0x0400;
const OBJ_ALIAS_REQUIRED: u16 =  0x1000;
const OBJ_BIG: u16 =             0x2000;
const OBJ_CONFORMING: u16 =      0x4000;
const OBJ_IOPL: u16 =            0x8000;
pub enum LXObjectRights {
    /// Rights of "code32" section
    ///  - READ
    ///  - EXEC
    CODE = 1,
    /// Rights of "data32"
    ///  - READ
    ///  - WRITE
    DATA = 2,
    /// Rights of "rdata32"
    ///  - READ
    RDATA = 3,
    BSS = 4,
    /// Rights of god object
    ///  - READ
    ///  - WRITE
    ///  - EXEC
    GOD = 5,
    /// Non-readable object
    SETTER = 7,
}
pub struct ObjectsTable {
    pub objects: Vec<Object>,
}
impl ObjectsTable {
    pub fn read<T: Read + Seek>(
        reader: &mut T,
        objtab: u64,
        count: u32,
    ) -> Result<ObjectsTable, Error> {
        let mut objects = Vec::<Object>::new();
        reader.seek(SeekFrom::Start(objtab))?;
        for _ in 0..count {
            let mut caught_obj = [0; 24];
            reader.read_exact(&mut caught_obj)?;
            objects.push(bytemuck::cast(caught_obj));
        }

        Ok(ObjectsTable { objects })
    }
}
