// Copyright (c) 2022 Charles M. Thompson
//
// This file is part of ApeCrunch.
//
// ApeCrunch is free software: you can redistribute it and/or modify it under
// the terms only of version 3 of the GNU General Public License as published
// by the Free Software Foundation
//
// ApeCrunch is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License
// for more details.
//
// You should have received a copy of the GNU General Public License along with
// ApeCrunch(in a file named COPYING).
// If not, see <https://www.gnu.org/licenses/>.

use directories::ProjectDirs;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Session {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl Session {
    pub fn new() -> Self {
        let qualifier = "org";
        let organisation = "Open Ape Shop";
        let application = "ApeCrunch";

        let dirs = ProjectDirs::from(qualifier, organisation, application).unwrap();

        return Self {
            config_dir: dirs.config_dir().to_owned(),
            data_dir: dirs.data_dir().to_owned(),
        };
    }

    pub fn new_test() -> Self {
        return Self {
            config_dir: Path::new("test/config").to_owned(),
            data_dir: Path::new("test/data").to_owned(),
        };
    }

    pub fn config_dir(&self, path: &Path) -> Self {
        return Self {
            config_dir: path.to_owned(),
            data_dir: self.data_dir.clone(),
        };
    }

    pub fn data_dir(&self, path: &Path) -> Self {
        return Self {
            config_dir: self.config_dir.clone(),
            data_dir: path.to_owned(),
        };
    }

    pub fn init(&self) -> Result<(), Box<dyn Error>> {
        // Create the directories if they don't exist
        if !self.config_dir.exists() {
            fs::create_dir_all(self.config_dir.as_path())?;
        }

        if !self.data_dir.exists() {
            fs::create_dir_all(self.data_dir.as_path())?;
        }

        // Create theme file if it doesn't exist
        if !self.get_theme_file_path().exists() {
            self.create_default_theme_file()?;
        }

        return Ok(());
    }

    pub fn purge(&self) -> Result<(), Box<dyn Error>> {
        // Delete the directories if they exist
        if self.config_dir.exists() {
            fs::remove_dir_all(self.config_dir.as_path())?;
        }

        // Delete the directories if they exist
        if self.data_dir.exists() {
            fs::remove_dir_all(self.data_dir.as_path())?;
        }

        return Ok(());
    }

    pub fn get_theme_file_path(&self) -> PathBuf {
        let mut file_path: PathBuf = self.config_dir.clone();

        file_path.push(Path::new(DEFAULT_THEME_TOML_NAME));

        return file_path;
    }

    pub fn create_default_theme_file(&self) -> Result<(), Box<dyn Error>> {
        let file_path = self.get_theme_file_path();

        let mut file = File::options()
            .read(false)
            .write(true)
            .create_new(true)
            .open(file_path.as_path())?;

        file.write_all(DEFAULT_THEME_TOML.as_bytes())?;

        return Ok(());
    }
}

static DEFAULT_THEME_TOML_NAME: &str = "theme.toml";

static DEFAULT_THEME_TOML: &str = r##"# Auto generated theme

shadow = false
borders = "simple"

[colors]
    background  = "#000000"
    view        = "#282a36"
    
    primary     = "#f8f8f2"
    secondary   = "#8be9fd"
    tertiary    = "#6272a4"

    highlight   = "#44475a"
"##;
