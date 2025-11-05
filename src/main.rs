use std::io::{Read, Seek};
use std::ops::Index;

mod exe;
mod exe286;
mod exe386;
mod types;

/// It will be Dynamic linked object later
///  - rustc 1.88.0 (6b00bc388 2025-06-23)
///  - bytemuck 1.24.0
fn main() -> std::io::Result<()> {
    let path = "D:\\TEST\\OS2\\SYSINST1.EXE";

    let ne = exe286::NeExecutableLayout::get(path)?;

    for (i, imp) in ne.imp_tab.iter().enumerate() {
        imp.imp_list.iter().enumerate().for_each(|(j, v)| {
            match v.ordinal {
                0 => println!("{}::{}", v.dll_name.to_string(), v.name.to_string()),
                _ => println!("{}::@{}", v.dll_name.to_string(), v.ordinal)
            }
        })
    }
    Ok(())
}
