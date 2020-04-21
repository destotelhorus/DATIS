//Source: https://stackoverflow.com/questions/37223741/how-can-i-take-input-from-either-stdin-or-a-file-if-i-cannot-seek-stdin
use std::fs;
use std::io::{self, Read, Seek, SeekFrom};

pub enum Input {
    File(fs::File),
    Stdin(io::Stdin),
}

impl Read for Input {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            Input::File(ref mut file) => file.read(buf),
            Input::Stdin(ref mut stdin) => stdin.read(buf),
        }
    }
}

impl Seek for Input {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match *self {
            Input::File(ref mut file) => file.seek(pos),
            Input::Stdin(_) => {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "not supported by stdin-input",
                ))
            },
        }
    }
}
