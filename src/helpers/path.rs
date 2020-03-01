use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{self, Error as IOError, ErrorKind};
use std::os::unix::fs::OpenOptionsExt;

use dirs;
use std::path::PathBuf;

fn config_dir() -> io::Result<PathBuf> {
    match dirs::config_dir() {
        Some(path) => Ok(path),
        None => Err(IOError::new(
            ErrorKind::NotFound,
            "Missing Home Directory from environment",
        )),
    }
}

#[cfg(any(unix))]
pub fn assets_dir() -> String {
    String::from("/usr/share/rustaman")
}

pub fn rustaman_config_dir() -> io::Result<PathBuf> {
    let mut path = config_dir()?;
    path.push("rustaman");

    if !path.exists() {
        fs::create_dir_all(path.to_str().unwrap())?;
    } else if !path.is_dir() {
        return Err(IOError::new(
            ErrorKind::InvalidData,
            format!("{} should be a directory", path.to_str().unwrap()),
        ));
    }
    Ok(path)
}

/// Return the path file,
/// Raise IOError in case of environment or permission error.
pub fn workspace(filename: &str) -> io::Result<PathBuf> {
    let mut path = rustaman_config_dir()?;
    path.push(filename);
    Ok(path)
}

pub fn write_file(filepath: &str, filecontent: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .mode(0o644)
        .write(true)
        .create(true)
        .truncate(true)
        .open(filepath)?;
    file.write_all(filecontent.as_bytes())?;
    Ok(())
}

pub fn read_file(filepath: &str) -> io::Result<String> {
    let mut file = File::open(filepath)?;
    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf)?;
    let res = String::from_utf8(buf).unwrap(); // UTF8 error will crash...
    Ok(res)
}
