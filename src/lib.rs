mod exe;
mod exe286;
mod exe386;
mod types;

pub enum ExecutableFormat {
    MZ,
    NE, // Windows/OS2 16-bit
    LE, // Windows 3.x VxD
    LX, // OS/2 32-bit
}

pub struct ExecutableInfo {
    pub format: ExecutableFormat,
    pub architecture: String,
    pub entry_point: Option<u32>,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
    //pub segments: Vec<Segment>,
    //pub resources: Vec<Resource>,
}

pub struct Import {
    pub module_name: String,
    pub function_name: Option<String>,
    pub ordinal: Option<u16>,
}

pub struct Export {
    pub name: String,
    pub ordinal: u16,
    pub address: u32,
}

#[cfg(test)]
mod tests {
    use crate::exe386;

    #[test]
    fn it_works() {
        let path = "D:\\TEST\\MS_OS220\\DOSCALL1.DLL";
        let layout = exe386::LinearExecutableLayout::get(path);

        match layout {
            Ok(res) => println!("{:?}", res.header),
            Err(e) => eprint!("{:?}", e),
        }
    }
}
