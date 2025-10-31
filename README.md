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

Linear Executable (`LE`) format is a try to make a universal format
of executables. Files linked as `LE` may contain 16-bit and 32-bit code
inside. Physical scopes inside called as "Objects". This is a not clearly segments
like in previous New executable files. One of segment may contain 32-bit code
but another segment may contain 16-bit code.

Each object in file must have been placed in special registered and allocated
space in memory. This space calls "Object Page".

Object page's size is a fixed, set in main header - value.
Usually for Intel x86 linked `LE` objects it equals `4096` but
any way it wolud be better if you look up at the filled header.

"Communication" between objects can be implemented through
the per-segment fixups. And the hardest structure of this file is 
a "Fixup Records Table". But only from there we can know imports and internal/external
relocation records.

Operating Systems used `LE` executables:
 - Microsoft Windows 3.x, (.386 Virtual xxx Drivers)
 - Microsoft Windows 9x, (.386/.vxd Virtual xxx Drivers)
 - IBM OS/2 2x (exactly ALL software)
 - ~Microsoft~ OS/2 2.0 (exactly all software)

System utilities what can produce/run Linear executables:
 - Watcom/Open Watcom linker
 - EMX linker and compiler?
 - DOS Extenders

Linear eXecutable format is a **standard** of OS/2 linked objects
and programs. All well-known OS/2 family operating systems are fully
built as LX executables. They are also contains 16-bit and 32-bit code
inside. And concept of Objects and Intel 286 "CallGates" are still here.

But `LX` objects are more difficult and extended instead of `LE` files.
And based on the "Undocumented Windows File Formats" book:

> [!NOTE]
> "The LE format is actually based on, or at least very similar to, the LX file format
used by OS/2 executables. In fact, all of the work in reverse-engineering the LE format
was based on information available on the LX format"

That's all.

Operating Systems used LX executables:
 - IBM OS/2 3.0
 - IBM OS/2 4x
 - eCOM Station
 - ArcaNoae ArcaOS

### In the end

This is not mostly idea to transfer .NET-based module to Rust.
This is a just library like a [goblin](https://github.com/m4d/goblin) crate
what may help you to resolve strange undocumented features of old software.

