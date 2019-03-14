use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use ini::Ini;
use sha1::Sha1;

use crate::errors::GitError;
use crate::fsutils;
use crate::repository::config::GitConfig;
use crate::repository::object::{GitObject, Serializable, Type};

pub mod config;
pub mod object;

#[derive(Debug)]
pub struct GitRepository {
    pub work_tree: PathBuf,
    pub git_dir: PathBuf,
    pub conf: GitConfig,
}

impl GitRepository {
    /// Creates a new Git repository at the given path
    pub fn create(path: &Path) -> Result<GitRepository, GitError> {
        let repo = GitRepository::new(path);

        GitRepository::create_work_tree(&repo, &path)?;

        repo.repo_dir(&path!("branches"), true);
        repo.repo_dir(&path!("objects"), true);
        repo.repo_dir(&path!("refs/tags"), true);
        repo.repo_dir(&path!("refs/heads"), true);

        GitRepository::create_repo_file(&repo, &path!("description"), "Unnamed repository; edit this file 'description' to name the repository.")?;
        GitRepository::create_repo_file(&repo, &path!("HEAD"), "ref: refs/heads/master")?;

        let default_config: GitConfig = Default::default();
        let config = repo.repo_file(&path!("config"))?;
        default_config.save(&config).unwrap();

        Ok(repo)
    }

    /// Create a new repository object.
    ///
    /// This method creates the object only in memory. If you want to create a new repository
    /// that is saved to the disk, use GitRepository::create instead
    pub fn new(path: &Path) -> GitRepository {
        let gitdir = path.join(&path!(".git"));
        let conf = Ini::new();

        GitRepository {
            work_tree: path.to_path_buf(),
            git_dir: gitdir,
            conf: GitConfig::new(conf),
        }
    }

    /// Load an existing repository
    pub fn load(path: &Path) -> Result<GitRepository, GitError> {
        let gitdir = path.join(path!(".git"));
        let config_path = gitdir.join(&path!("config"));
        let conf = Ini::load_from_file(&config_path)
            .map_err(|e| {
                GitError::GenericError(format!("Unable to load git config for repo: {}, {}", config_path.to_str().unwrap(), e.to_string()))
            })?;

        Ok(GitRepository {
            work_tree: path.to_path_buf(),
            git_dir: gitdir,
            conf: GitConfig::new(conf),
        })
    }

    /// Tries to find a git repository initiating from the current working directory
    pub fn find() -> Result<GitRepository, GitError> {
        let cwd = fsutils::cwd()?;
        let mut it = Some(cwd);

        while it.is_some() {
            let current = it.as_ref().unwrap();
            let path = current.join(path!(".git"));
            if path.exists() {
                return Ok(GitRepository::load(&current)?);
            }

            it = it.unwrap().parent().map(|p| p.to_path_buf());
        }

        Err(GitError::GenericError("Git repository could not be found".to_owned()))
    }

    /// Create the repo's base working directory.
    ///
    /// Throws error if the directory already exists and it is not empty, or if the given path is not a directory.
    fn create_work_tree(repo: &GitRepository, path: &Path) -> Result<(), GitError> {
        if repo.work_tree.exists() {
            if !repo.work_tree.is_dir() {
                return Err(GitError::PathError("Specified path is not a directory".to_owned(), repo.work_tree.clone()));
            } else if !fsutils::is_dir_empty(path) {
                return Err(GitError::PathError("Specified directory is not empty".to_owned(), repo.work_tree.clone()));
            }
        } else {
            create_dir_all(&repo.work_tree).map_err(|_| {
                GitError::PathError("Could not create directory".to_owned(), repo.work_tree.clone())
            })?;
        }
        Ok(())
    }

    /// Creates a new file in the .git directory with the specified content
    fn create_repo_file(repo: &GitRepository, path: &Path, contents: &str) -> Result<(), GitError> {
        let file = repo.repo_file(path)?;
        fsutils::create_file_and_write(&file, contents)
    }
}

impl GitRepository {
    /// Compute a path under repo's gitdir
    pub fn repo_path(&self, path: &Path) -> PathBuf {
        self.git_dir.join(path)
    }

    pub fn repo_file(&self, path: &Path) -> Result<PathBuf, GitError> {
        let full_path = self.repo_path(path);
        let parent = full_path.parent().unwrap();
        create_dir_all(parent)
            .map_err(|_| {
                GitError::PathError(
                    "Could not create directories: {}".to_owned(),
                    parent.to_path_buf(),
                )
            })?;
        Ok(full_path)
    }

    pub fn repo_dir(&self, path: &Path, mkdir: bool) -> Option<PathBuf> {
        let path = self.repo_path(path);

        if path.exists() {
            if path.is_dir() {
                return Some(path.clone());
            } else {
                return None;
            }
        } else if mkdir {
            create_dir_all(&path).unwrap();
            return Some(path.clone());
        }

        None
    }

    pub fn object_find(&self, name: &str, _format: &Type) -> String {
        name.to_owned()
    }

    pub fn read_object(&self, sha: &str) -> Result<GitObject, GitError> {
        let dir = &sha[0..2];
        let rest = &sha[2..];

        let object = self.repo_file(&path!("objects", dir, rest))?;

        let file = File::open(&object).map_err(|_| {
            GitError::PathError("Could not open file".to_owned(), object.to_path_buf())
        })?;

        let mut decompressed = Vec::new();
        ZlibDecoder::new(file).read_to_end(&mut decompressed).map_err(|e| {
            GitError::GenericError(format!("Could not read object data: {}", e.to_string()))
        })?;

        let x = decompressed.iter().position(|b| b == &b' ').unwrap();
        let y = decompressed.iter().position(|b| b == &b'\x00').unwrap();
        let bytes: &[u8] = decompressed.as_ref();

        let format = &bytes[0..x];
        let data = &bytes[y + 1..];

        let size: usize = String::from_utf8(bytes[x..y].to_vec()).unwrap().trim().parse().unwrap();
        if size != decompressed.len() - y - 1 {
            return Err(GitError::ObjectError(format!("Malformed object {0}: bad length", sha)));
        }

        let object_type = Type::deserialize(format);
        Ok(GitObject::new(object_type, data))
    }

    pub fn write_object(repo: &GitRepository, object: &GitObject) -> Result<String, GitError> {
        let mut data = object.serialize().to_vec();
        let mut result = Vec::new();
        result.append(&mut object.object_type.serialize().to_vec());
        result.push(b' ');
        result.append(&mut data.len().to_string().to_ascii_lowercase().as_bytes().to_vec());
        result.push(0 as u8);
        result.append(&mut data);

        let sha = Sha1::from(&result).hexdigest();
        let path = repo.repo_file(&path!("objects", &sha[0..2], &sha[2..]))?;

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(1));
        encoder.write_all(&mut result)
            .and(encoder.finish())
            .and_then(|compressed| {
                let file = File::create(path)?;
                Ok((compressed, file))
            })
            .and_then(|(compressed, mut file)| {
                file.write_all(&compressed)
            })
            .and(Ok(sha.to_owned()))
            .map_err(|e| {
                GitError::GenericError(format!("Unable to compress and save object data: {} - {}", sha, e.to_string()))
            })
    }
}