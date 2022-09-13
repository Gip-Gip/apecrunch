//! Manages config files and various settings.
//!

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

use crate::variable::VarTable;
use directories::ProjectDirs;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

/// Serializable version of session. For creating session.toml.
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTOML {
    pub decimal_places: Option<u32>,
    pub history_depth: Option<u32>,
}

/// All semi-global settings and variables that are needed for the session.
///
#[derive(Debug, Clone)]
pub struct Session {
    /// Path to the config directory.
    pub config_dir: PathBuf,
    /// Path to the data directory.
    pub data_dir: PathBuf,
    /// Number of decimal places to render.
    pub decimal_places: u32,
    /// Number of history entries to render.
    pub history_depth: u32,
    /// Variables stored in the session
    pub vartable: VarTable,
}

impl Session {
    /// Creates a new (default) session.
    ///
    /// Sets config and data directory to system defaults.
    ///
    pub fn new() -> Self {
        let qualifier = "org";
        let organisation = "Open Ape Shop";
        let application = "ApeCrunch";

        let dirs = ProjectDirs::from(qualifier, organisation, application).unwrap();

        Self {
            config_dir: dirs.config_dir().to_owned(),
            data_dir: dirs.data_dir().to_owned(),
            decimal_places: DEFAULT_DECIMAL_PLACES,
            history_depth: DEFAULT_HISTORY_DEPTH,
            vartable: VarTable::new(),
        }
    }

    /// Creates a session that is safe for cargo test.
    ///
    /// Config and data files are stored in test/config/ and test/data/ respectively.
    ///
    pub fn _new_test() -> Self {
        Self {
            config_dir: Path::new("test/config").to_owned(),
            data_dir: Path::new("test/data").to_owned(),
            decimal_places: DEFAULT_DECIMAL_PLACES,
            history_depth: DEFAULT_HISTORY_DEPTH,
            vartable: VarTable::new(),
        }
    }

    /// Initialize a session, reading and, if necissary, creating, various config files needed for basic operation.
    ///
    pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
        // Create the directories if they don't exist.
        if !self.config_dir.exists() {
            fs::create_dir_all(self.config_dir.as_path())?;
        }

        if !self.data_dir.exists() {
            fs::create_dir_all(self.data_dir.as_path())?;
        }

        // Create theme file if it doesn't exist.
        let theme_file_path = self.get_theme_file_path();
        if !theme_file_path.exists() {
            self.create_default_config_file(&theme_file_path, DEFAULT_THEME_TOML)?;
        }

        // Create session config file if it doesn't exist.
        let session_config_file_path = self.get_session_config_file_path();
        if !session_config_file_path.exists() {
            self.create_default_config_file(&session_config_file_path, DEFAULT_SESSION_TOML)?;
        }

        // Load the config file.
        let mut session_config_file = File::open(session_config_file_path)?;
        let mut session_config_data = Vec::<u8>::new();

        session_config_file.read_to_end(&mut session_config_data)?;

        let session_toml: SessionTOML = toml::from_slice(&session_config_data)?;

        // Apply the config file to the session.
        if let Some(decimal_places) = session_toml.decimal_places {
            self.decimal_places = decimal_places;
        }

        if let Some(history_depth) = session_toml.history_depth {
            self.history_depth = history_depth;
        }

        Ok(())
    }

    /// Purge all config and data files, currently only used in cargo test.
    ///
    pub fn _test_purge(&self) -> Result<(), Box<dyn Error>> {
        // Delete the directories if they exist.
        if self.config_dir.exists() {
            fs::remove_dir_all(self.config_dir.as_path())?;
        }

        // Delete the directories if they exist.
        if self.data_dir.exists() {
            fs::remove_dir_all(self.data_dir.as_path())?;
        }

        Ok(())
    }

    /// Get the path to the theme file, given the default theme filename.
    ///
    pub fn get_theme_file_path(&self) -> PathBuf {
        let mut file_path: PathBuf = self.config_dir.clone();

        file_path.push(Path::new(DEFAULT_THEME_TOML_NAME));

        file_path
    }

    /// Get the path to the session config file, given the default config filename.
    ///
    pub fn get_session_config_file_path(&self) -> PathBuf {
        let mut file_path: PathBuf = self.config_dir.clone();

        file_path.push(Path::new(DEFAULT_SESSION_TOML_NAME));

        file_path
    }

    /// Create a given default config file given a path and the contents of the file in a string.
    ///
    /// Returns file i/o errors if any. **Will refuse to overwrite existing files.**
    ///
    pub fn create_default_config_file(
        &self,
        path: &Path,
        contents: &str,
    ) -> Result<(), Box<dyn Error>> {
        let mut file = File::options()
            .read(false)
            .write(true)
            .create_new(true)
            .open(path)?;

        file.write_all(contents.as_bytes())?;

        Ok(())
    }
}

/// Default number of decimal places to render. Does not affect precision of calculations.
pub const DEFAULT_DECIMAL_PLACES: u32 = 6;

/// Default number of history entries to render.
pub const DEFAULT_HISTORY_DEPTH: u32 = 1000;

/// Default filename of the session config file.
pub static DEFAULT_SESSION_TOML_NAME: &str = "session.toml";

/// Default filename of the theme config file.
pub static DEFAULT_THEME_TOML_NAME: &str = "theme.toml";

/// Contents of the default session config file.
pub static DEFAULT_SESSION_TOML: &str = r##"# Auto generated session config

decimal_places = 6
history_depth = 1000
"##;

/// Contents of the default theme config file. Kinda going for a darkula theme here
pub static DEFAULT_THEME_TOML: &str = r##"# Auto generated theme

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
