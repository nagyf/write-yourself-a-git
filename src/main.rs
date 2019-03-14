#[macro_use]
extern crate clap;
extern crate flate2;
extern crate ini;

use std::path::Path;

use clap::{App, ArgMatches};

use crate::errors::GitError;
use crate::repository::GitRepository;
use crate::repository::object::{GitObject, Serializable};
use crate::repository::object::Type;

#[macro_use]
pub mod macros;
pub mod fsutils;
pub mod errors;
pub mod repository;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(matches) = matches.subcommand_matches("init") {
        init(matches);
    } else if let Some(matches) = matches.subcommand_matches("cat-file") {
        cat_file(matches).unwrap();
    } else if let Some(matches) = matches.subcommand_matches("hash-object") {
        hash_object(matches).unwrap();
    }
}

fn init(matches: &ArgMatches) {
    if matches.is_present("path") {
        let repo_name = matches.value_of("path").unwrap();
        let result = GitRepository::create(Path::new(repo_name));
        if result.is_err() {
            println!("{}", result.unwrap_err());
        }
    } else {
        println!("{}", matches.usage());
    }
}

fn cat_file(matches: &ArgMatches) -> Result<(), GitError> {
    let object = matches.value_of("object").unwrap();
    let object_type: Type = Type::deserialize(matches.value_of("type").unwrap().as_bytes());
    GitRepository::load(&path!("."))
        .and_then(|repo| {
            let object = repo.object_find(object, &object_type);
            repo.read_object(&object)
        })
        .and_then(|obj| {
            String::from_utf8(obj.serialize().to_vec())
                .map_err(|e| GitError::GenericError(format!("Error converting object data to string: {}", e.to_string())))
        })
        .and_then(|object_as_string| {
            println!("{}", object_as_string);
            Ok(())
        })
}

fn hash_object(matches: &ArgMatches) -> Result<(), GitError> {
    let mut repo = None;
    if matches.is_present("write") {
        GitRepository::find()
            .and_then(|found| {
                repo = Some(found);
                Ok(())
            })?;
    }

    let object_type = Type::deserialize(matches.value_of("type").unwrap().as_bytes());
    let path = path!(matches.value_of("path").unwrap());
    let content = fsutils::read_content(&path)?;
    let object = GitObject::new(object_type, &content);

    if let Some(repo) = repo {
        GitRepository::write_object(&repo, &object)
            .and_then(|sha| {
                println!("{}", sha);
                Ok(())
            })?;
    }

    Ok(())
}