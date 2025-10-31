use std::io::{Read, Seek};

mod exe;
mod exe286;
mod exe386;

/// It will be Dynamic linked object later
///  - rustc 1.88.0 (6b00bc388 2025-06-23)
///  - bytemuck 1.24.0
fn main() {
    let path = "D:\\TEST\\Windows2.1\\CALC.EXE";

    let ne = exe286::NeExecutableLayout::get(path);

}
