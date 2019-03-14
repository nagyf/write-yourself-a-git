use std::env;
use std::fs::{File, read_dir};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::io::Read;

use crate::errors::GitError;

pub fn is_dir_empty(path: &Path) -> bool {
    read_dir(&path).unwrap().map(|_| 1).sum::<i32>() == 0
}

pub fn create_file_and_write(path: &Path, contents: &str) -> Result<(), GitError> {
    let mut file = File::create(path).map_err(|_e| {
        GitError::PathError("Could not create file".to_owned(), path.to_path_buf())
    })?;

    file.write_all(contents.as_bytes()).map_err(|_e| {
        GitError::PathError("Could not write file".to_owned(), path.to_path_buf())
    })?;

    file.write(b"\n").map_err(|e| e.to_string()).map_err(|_e| {
        GitError::PathError("Could not write file".to_owned(), path.to_path_buf())
    })?;

    Ok(())
}

pub fn read_content(path: &Path) -> Result<Vec<u8>, GitError> {
    let mut content = Vec::new();
    File::open(&path)
        .and_then(|mut file| {
            file.read_to_end(&mut content)
        })
        .and(Ok(content))
        .map_err(|e| {
            GitError::PathError(format!("Could not read file {}", e.to_string()), path.to_path_buf())
        })
}

/// Returns the current working directory
pub fn cwd() -> Result<PathBuf, GitError> {
    env::current_dir().map_err(|_| {
        GitError::GenericError("Cannot open current working directory!".to_owned())
    })
}