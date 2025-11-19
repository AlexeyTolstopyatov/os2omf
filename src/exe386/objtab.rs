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
// OBJ_READABLE        0x0001
// OBJ_WRITEABLE       0x0002
// OBJ_EXECUTABLE      0x0004
// OBJ_RESOURCE        0x0008
// OBJ_DISCARDABLE     0x0010
// OBJ_SHARABLE        0x0020
// OBJ_HAS_PRELOAD     0x0040
// OBJ_HAS_INVALID     0x0080
// OBJ_PERM_SWAPPABLE  0x0100  /* LE */
// OBJ_HAS_ZERO_FILL   0x0100  /* LX */
// OBJ_PERM_RESIDENT   0x0200
// OBJ_PERM_CONTIGUOUS 0x0300  /* LX */
// OBJ_PERM_LOCKABLE   0x0400
// OBJ_ALIAS_REQUIRED  0x1000
// OBJ_BIG             0x2000
// OBJ_CONFORMING      0x4000
// OBJ_IOPL            0x8000
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
