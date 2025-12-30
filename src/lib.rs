//! This crate provides format specifications of legacy
//! file formats what uses mostly in IBM OS/2 different versions and revisions
//!
//! Information which represented here bases mostly on official documents
//! by IBM and Microsoft, but some of the facts from there are invalid and fixed.
//! 
//! ### Support
//!
//! This crate supports formats of executables:
//!  - `MZ (mod exe)` DOS 16-bit executables
//!  - `NE (mod exe286)` Windows 1.x-3x Protected-Mode 16-bit executables
//!  - `LE (mod exe386)` Microsoft OS/2 2.0+ and Windows 9x VxDs 16-32-bit
//!  - `LX (mod exe386)` IBM OS/2 2.0-4.5 16-32-bit executables
//!
//!
//! ### Issues
//! List what has written here is temporary, I hope.
//! I really want to fix all known problems and specially warn you about most serious of them.
//! 
//! - Crate works correctly only with `LittleEndian` linked files;
//! - Some of the structures are undocumented;
//! - No correct data-container for values (the worst for cross-platform compilation);
//! - No support for VxD files yet (specific VxD structures);
//! - No support for resources blocks. (can't read resource table yet)
//!

/// 16-bit DOS Executables
pub mod exe;
/// Segmented 16-bit New Executables 
pub mod exe286;
/// Microsoft-IBM 16-32-bit Linear Executables
pub mod exe386;
/// Support of specific types
pub mod types;

#[cfg(test)]
mod exe_386_tests {
    use crate::exe386;

    #[test]
    fn e386_header() {
        let path = "D:\\TEST\\MS_OS220\\DOSCALL1.DLL";
        let layout = exe386::LinearExecutableLayout::get(path);

        match layout {
            Ok(res) => assert!(true, "{:?}", res.header),
            Err(e) => assert!(false, "{:?}", e),
        }
    }

    #[test]
    fn e386_enttab() {
        let path = "D:\\TEST\\MS_OS220\\DOSCALL1.DLL";
        let layout = exe386::LinearExecutableLayout::get(path);

        match layout {
            Ok(res) => {
                assert!(true, "{:?}", res.entry_table.bundles);
            },
            Err(e) => assert!(false, "{:?}", e),
        }
    }
    #[test]
    fn e386_imports() {
        let path = "D:\\TEST\\ARCA\\BDCALLS.DLL";
        let layout = exe386::LinearExecutableLayout::get(path);

        match layout {
            Ok(res) => {
                assert!(true, "{:?}", res.import_table.imports());
            },
            Err(e) => assert!(false, "{:?}", e),
        }
    }
}
