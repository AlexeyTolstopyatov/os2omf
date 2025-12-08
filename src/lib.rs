pub mod exe;
pub mod exe286;
pub mod exe386;
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
