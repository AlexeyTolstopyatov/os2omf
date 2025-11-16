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
    let exec = exe386::LinearExecutableLayout::read("D:\\TEST\\ARCA\\BDCALLS.DLL")?;


    Ok(())
}
