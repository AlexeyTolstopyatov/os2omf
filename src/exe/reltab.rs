pub struct RelocationRecordsTable {
    relocations: Vec<FarPointer>
}
pub struct FarPointer {
    pub segment: u16,
    pub offset: u16,
}
