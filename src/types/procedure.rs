use crate::types::PascalString;

pub(crate) fn format_windbg(mod_str: &PascalString, proc_str: &PascalString) -> String {
    format!("{}!{}", mod_str.to_string(), proc_str.to_string())
}

pub(crate) fn format_rs(mod_str: &PascalString, proc_str: &PascalString) -> String {
    format!("{}::{}", mod_str.to_string(), proc_str.to_string())
}