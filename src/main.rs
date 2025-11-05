use std::io::{Read, Seek};

mod exe;
mod exe286;
mod exe386;
mod types;

/// It will be Dynamic linked object later
///  - rustc 1.88.0 (6b00bc388 2025-06-23)
///  - bytemuck 1.24.0
fn main() -> std::io::Result<()> {
    let path = "D:\\TEST\\OS2\\SYSINST2.EXE";

    let ne = exe286::NeExecutableLayout::get(path)?;

    Ok(())
}
