use crate::exe286::segtab::DllImport;

mod exe;
mod exe286;
mod exe386;
mod types;

pub enum TargetObject {
    MZModule,
    NEModule,
    LEModule,
    LXModule
}

/// It will be Dynamic linked object later
///  - rustc 1.88.0 (6b00bc388 2025-06-23)
///  - bytemuck 1.24.0
fn main() -> std::io::Result<()> {
    let exec = exe386::LinearExecutableLayout::from("D:\\TEST\\ARCA\\BDCALLS.DLL")?;
    
    println!("imports_cnt={}", exec.import_table.imports().len());
    for i in exec.import_table.imports() {
        match i {
            exe386::imptab::DllImport::ImportName(name) => {
                println!("{}::{}", name.module_name.to_string(), name.import_name.to_string())
            }
            exe386::imptab::DllImport::ImportOrdinal(ordinal) => {
                println!("{}::@{}", ordinal.module_name.to_string(), ordinal.import_ordinal);
            },
        }
    }
    Ok(())
}
