/* anevicon: The most powerful UDP-based load generator, written in Rust.
 * Copyright (C) 2019  Temirkhan Myrzamadi <gymmasssorla@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 * For more information see <https://github.com/Gymmasssorla/anevicon>.
 */

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io;
use std::num::NonZeroUsize;
use std::path::Path;

use rand::{thread_rng, RngCore};

use super::config::ArgsConfig;

pub fn construct_packet(args_config: &ArgsConfig) -> Result<Vec<u8>, ReadPacketError> {
    // If our user specified a file, then use file content as a packet.
    // Otherwise, generate a random set of bytes to use as a packet.
    if let Some(ref filename) = args_config.file {
        read_packet(filename)
    } else {
        Ok(random_packet(args_config.length))
    }
}

pub fn read_packet<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, ReadPacketError> {
    let content = fs::read(path).map_err(|error| ReadPacketError::ReadFailed(error))?;

    if content.len() == 0 {
        return Err(ReadPacketError::ZeroSize);
    }

    Ok(content)
}

#[derive(Debug)]
pub enum ReadPacketError {
    ReadFailed(io::Error),
    ZeroSize,
}

impl Display for ReadPacketError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            ReadPacketError::ReadFailed(error) => write!(fmt, "{}", error),
            ReadPacketError::ZeroSize => write!(fmt, "Zero packet size"),
        }
    }
}

impl Error for ReadPacketError {}

pub fn random_packet(length: NonZeroUsize) -> Vec<u8> {
    // Create a sending buffer without an unnecessary initialization
    // because we'll fill this buffer with random values next.
    let mut buffer = Vec::with_capacity(length.get());
    unsafe {
        buffer.set_len(length.get());
    }

    thread_rng().fill_bytes(buffer.as_mut_slice());
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;

    use tempfile::NamedTempFile;

    #[test]
    #[should_panic(expected = "Zero packet size")]
    fn test_read_zero_file() {
        let temp = NamedTempFile::new().expect("Cannot create a temporary file");

        if let Err(ReadPacketError::ZeroSize) = read_packet(temp.path()) {
            panic!("Zero packet size");
        } else {
            panic!("Must return the 'ZeroSize' error");
        }
    }

    #[test]
    fn test_read_valid_file() {
        let mut temp = NamedTempFile::new().expect("Cannot create a temporary file");

        let content = vec![26; 4096];
        temp.write_all(&content).unwrap();

        let read_file = read_packet(temp.path()).expect("Cannot read a temporary file");
        assert_eq!(read_file, content);
    }

    #[test]
    fn generates_random_packet() {
        let length = unsafe { NonZeroUsize::new_unchecked(35684) };
        let buffer = random_packet(length);

        // Check that we've got the correctly length and capacity
        assert_eq!(buffer.len(), length.get());
        assert!(buffer.capacity() >= length.get());
    }
}
