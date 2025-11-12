use std::io;
use std::io::Read;

pub(crate) trait Readable<T> {
    fn read<R: Read>() -> io::Result<T>;
}