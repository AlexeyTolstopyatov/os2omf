### `os2omf` | OS/2 Object Module Formats

This is a crate what supports legacy undocumented 16/32 bit executables formats.
Earlier in the end of 80s times Microsoft Windows 1.0 and IBM PC-DOS could run in 
Intel x86 (Intel 286+) protected-mode. 

In this description you can't see explicit notations of OS/2 but
all described and documented/implemented formats in this crate _mostly uses by IBM and OS/2 family_

> [!WARNING]
> All file formats what have represented here are having one hard limit
> They are **applies to Intel x86 byte ordering** and only for it.
> Don't try to find OS/2 for PowerPC and parse one of component using this crate!

### `os2omf::exe` | Clear DOS x86 real-mode executables

This module represents structure of first segmented programs what could run
in the clear DOS. They are `MZ`-Executables in the standard understanding.
Mark Zbikowski executables or `MZ` are runs so close to physical devices 
what header specification required manually set a maximum and minimum allocation space.

The most famous field `e_lfanew: u32` what points to next protected-mode executable structure
was empty or zeroed in clear DOS programs. 

Operating Systems which uses `MZ` executables - 
 - Free-DOS
 - PC-DOS
 - MS-DOS
 - Novel-DOS
 - DR-DOS
 - etc...

Except the BW-DOS what uses specially shorten format of `MZ` executables.
BW-DOS applications are seriously different.

### `os2omf::exe286` | First x86 protected-mode executables

Segmented or "New" executables format was a Microsoft format for binaries which
runs after Intel context swithes. Segmeted memory model what still uses in Intel 286
protected mode is a base of programs and libraries linked by this specification.

> [!INFO]
> This module like mostly bases on [win16ne](https://github.com/qnighy/win16ne/) by 
> [qnighy](https://github.com/qnighy/) and my [Sunflower](https://github.com/AlexeyTolstopyatov/SunFlower.Plugins/) plugin 

Operating Systems which uses `NE` executables
 - Mutlitasking MS-DOS 4.0+;
 - IBM OS/2 1x; (exactly all software)
 - Microsoft Windows 1.x-3.x; (exactly all software)
 - Microsoft Windows 9x; (Userland drivers `.DRV` and old fonts `.FON`)

### `os2omf::exe386` | IBM and Microsoft tries to make universal object

Undocumented formats in this crate are really the same in most scopes
and I still can't believe this is real. 


### In the end

This is not mostly idea to transfer .NET-based module to Rust.
This is a just library like a [goblin](https://github.com/m4d/goblin) crate
what may help you to resolve strange undocumented features of old software.

