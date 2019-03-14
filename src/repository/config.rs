use std::fmt::{Debug, Error, Formatter};

use ini::Ini;
use std::path::Path;

pub struct GitConfig {
    pub conf: Ini
}

impl GitConfig {
    pub fn new(ini: Ini) -> GitConfig {
        GitConfig {
            conf: ini
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        self.conf.write_to_file(path).map_err(|e| e.to_string())
    }
}

impl Debug for GitConfig {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for (key, values) in self.conf.iter() {
            writeln!(f, "[{}]", key.as_ref().unwrap())?;
            for (key, value) in values {
                writeln!(f, "{} = {}", key, value)?;
            }
        }

        Ok(())
    }
}

impl Default for GitConfig {
    fn default() -> Self {
        let mut conf = Ini::new();
        conf.with_section(Some("core".to_owned()))
            .set("repositoryformatversion", "0")
            .set("filemode", "false")
            .set("bare", "false");

        GitConfig {
            conf
        }
    }
}