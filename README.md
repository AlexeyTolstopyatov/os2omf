### `os2omf` (OS/2 Object Module Formats)

This crate provides format specifications of legacy
file formats what uses mostly in IBM OS/2 different versions and revisions

Information which represented here bases mostly on official documents
by IBM and Microsoft, but some of the facts from there are invalid and fixed.

### Support

This crate supports formats of executables:
 - `MZ` DOS 16-bit executables 
 - `NE` Windows 1.x-3x Protected-Mode 16-bit executables
 - `LE` Microsoft OS/2 2.0+ and Windows 9x VxDs 16-32-bit
 - `LX` IBM OS/2 2.0-4.5 16-32-bit executables

### Quick start

The bad thing of this crate: no common module API.
You must know the target object.

Requirements for crate:
 - Rust 1.92 (stable)

This is an example "how to read windows new executable?" 

```rust
use os2omf::exe286::NewExecutableLayout;

pub fn main() -> Result<()> {
    let file_str = "put here Windows 3.1 app/dll path";
    let layout = NewExecutableLayout::get(file_str)?;
    //
    // Define flags of target module
    //
    println!("Target object: {}", file_str);
    println!("{:?}", layout.new_header.module_flags());
    
    // 
    // Define imports of target module
    //
    println!("Uses");
    for segment in layout.imp_tab {
        println!("Segment #{}", segment.seg_number);
        for i in segment.imp_list {
            println!("{}!{} at 0x{:x}", i.dll_name, i.name, i.file_pointer);
        }
    }
    //
    // Use other module descriptions to find anything
    // what you want.
    //
}
```

### History?

See [ADVANCED](ADVANCED.md) information file.

### Issues
List what has written here is temporary, I hope.
I really want to fix all known problems and specially warn you about most serious of them.

 - Crate works correctly only with `LittleEndian` linked files;
 - Some of the structures are undocumented;
 - No correct data-container for values (the worst for cross-platform compilation);
 - No support for VxD files (specific VxD structures);
 - No support for resources blocks. (can't read resource table yet)

### In the end

This is not mostly idea to rewrite .NET-based module into Rust library.
This is a just crate like a [goblin](https://github.com/m4d/goblin) ~~but worse~~
what may help you to resolve strange undocumented features of old software.
