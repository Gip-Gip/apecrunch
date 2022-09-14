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

use crate::parser::Token;
use crate::variable::VarTable;
use directories::ProjectDirs;
use lazy_static::*;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use std::cmp::Ordering;
use uuid::Uuid;

/// Versions of history files that this version of apecrunch is compatible with
///
pub const HISTORY_COMPAT_VERS: [&str; 2] = ["0.0.2", "0.0.3"];

/// Individual history entry retaining it's UUID, parser tokens, and textual rendition.
///
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HistoryEntry {
    /// UUID of the entry.
    entry_uuid: Uuid,
    /// Parser tokens of the entry, basically the entire expression parsed down into it's most basic form
    expression: Token,
    /// Rendition of the entry's expression at the time of calculation
    rendition: String,
}

impl HistoryEntry {
    /// Creates a new entry struct from parser tokens and the desired amount of decimal places to render.
    ///
    pub fn new(expression: &Token, decimal_places: u32) -> Self {
        let entry_uuid = Uuid::new_v4();

        Self {
            entry_uuid: entry_uuid,
            expression: expression.clone(),
            rendition: expression.to_string(decimal_places),
        }
    }

    /// Converts the entry to a string.
    ///
    pub fn to_string(&self) -> String {
        self.rendition.clone()
    }

    /// Renders the entry without an equal sign, nor everything right of it.
    ///
    /// Just returns the expression unmodified if there is no equal sign.
    ///
    pub fn render_without_equality(&self, decimal_places: u32) -> String {
        if let Token::Equality(left, _right) = &self.expression {
            return left.to_string(decimal_places);
        }

        self.expression.to_string(decimal_places)
    }
}

/// Layout of history bincodes when serializing a session.
///
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HistoryBincode {
    /// ApeCrunch Version in X.X.X format.
    pub version: String,
    /// Start of the session, in seconds since unix epoch.
    pub session_start: u64,
    /// Session UUID.
    pub session_uuid: Uuid,
    /// Decimal places visible when rendering numbers.
    pub decimal_places: u32,
    /// Session VarTable, all of the variables stored in the session
    pub session_vartable: VarTable,
    /// Vector containing all of the previous history entries.
    pub entries: Vec<HistoryEntry>,
}

impl HistoryBincode {
    /// Read an lz4_flex-compressed bincode from a slice and return a deserialized HistoryBincode
    ///
    pub fn from_slice(slice: &[u8]) -> Result<Self, Box<dyn Error>> {
        let uncompressed_data = lz4_flex::block::decompress_size_prepended(slice)?;

        Ok(bincode::deserialize(&uncompressed_data)?)
    }

    /// Serialize a HistoryBincode into an lz4_flex-compressed bincode, stored in a Vec<u8>
    ///
    /// **Note** that hopefully the uncompressed data is less that 4gb. If I'm correct lz4_flex uses a u32 for storing the uncompressed size of data,
    /// though I doubt this will become a problem unless you leave the same ApeCrunch instance open for a couple thousand years...
    ///
    pub fn to_vec(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(lz4_flex::block::compress_prepend_size(&bincode::serialize(
            &self,
        )?))
    }
}

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
    /// ApeCrunch Version in X.X.X format.
    pub version: String,
    /// Start of the session, in seconds since unix epoch.
    pub session_start: u64,
    /// Session UUID.
    pub session_uuid: Uuid,
    /// Vector containing all of the previous history entries.
    pub entries: Vec<HistoryEntry>,
    /// Path to the history file
    pub history_file_path: PathBuf,
    /// Previous entries found in previous sessions
    pub previous_entries: Vec<HistoryEntry>,
}

impl Session {
    /// Creates a new (default) session.
    ///
    /// Sets config and data directory to system defaults.
    ///
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let qualifier = "org";
        let organisation = "Open Ape Shop";
        let application = "ApeCrunch";

        let dirs = ProjectDirs::from(qualifier, organisation, application).unwrap();

        let session_uuid = Uuid::new_v4();
        let data_dir = dirs.data_dir().to_owned();
        let mut history_file_path = data_dir.clone();

        history_file_path.push(format!("history-{}.bincode.lz4", session_uuid));

        // Should not happen, but in the 2^128 chance that it does...
        if history_file_path.exists() {
            panic!(
                "Random file name generation failed! File {} already exists!",
                history_file_path.to_str().unwrap()
            );
        }

        Ok(Self {
            config_dir: dirs.config_dir().to_owned(),
            data_dir: data_dir,
            decimal_places: DEFAULT_DECIMAL_PLACES,
            history_depth: DEFAULT_HISTORY_DEPTH,
            session_start: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            session_uuid: session_uuid,
            version: crate::VERSION.to_string(),
            previous_entries: Vec::<HistoryEntry>::new(),
            entries: Vec::<HistoryEntry>::new(),
            history_file_path: history_file_path,
            vartable: VarTable::new(),
        })
    }

    /// Creates a session that is safe for cargo test.
    ///
    /// Config and data files are stored in test/config/ and test/data/ respectively.
    ///
    pub fn _new_test() -> Result<Self, Box<dyn Error>> {
        let data_dir = Path::new("test/data").to_owned();

        let session_uuid = Uuid::new_v4();
        let mut history_file_path = data_dir.clone();

        history_file_path.push(format!("history-{}.bincode.lz4", session_uuid));

        Ok(Self {
            config_dir: Path::new("test/config").to_owned(),
            data_dir: data_dir,
            decimal_places: DEFAULT_DECIMAL_PLACES,
            history_depth: DEFAULT_HISTORY_DEPTH,
            session_start: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            session_uuid: session_uuid,
            version: crate::VERSION.to_string(),
            previous_entries: Vec::<HistoryEntry>::new(),
            entries: Vec::<HistoryEntry>::new(),
            history_file_path: history_file_path,
            vartable: VarTable::new(),
        })
    }

    /// Initialize a session, reading and, if necissary, creating, various config files needed for basic operation.
    ///
    pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
        // Regex definitions for correctly identifying files
        lazy_static! {
            static ref HISTORY_FILE_RE: Regex =
                Regex::new(r"(.*)(history\-)(.+)(\.bincode\.lz4)").unwrap();
        }

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

        // Load all previous history files

        let mut previous_bincodes = Vec::<HistoryBincode>::new();

        // Go through each file in the session's data directory...
        for entry in fs::read_dir(&self.data_dir)? {
            let path = entry?.path().as_path().to_owned();

            let file_name = path.to_str().unwrap_or("");

            // And if the file name matches the regex...
            if HISTORY_FILE_RE.is_match(&file_name) {
                // Load it!
                let data = fs::read(&path)?;
                let history_bincode = match HistoryBincode::from_slice(&data) {
                    Ok(bincode) => bincode,
                    Err(_e) => {
                        // If our history file can't be serialized, print an error and move on...
                        eprintln!(
                            "History file {} corrupt or incompatible, not loading...",
                            file_name
                        );
                        continue;
                    }
                };

                // If our bincode version is compatible...
                if HISTORY_COMPAT_VERS.contains(&history_bincode.version.as_str()) {
                    previous_bincodes.push(history_bincode);
                }
                // Otherwise print an error...
                else {
                    eprintln!("History file version \"{}\" incompatible with apecrunch version {}, not loading...", history_bincode.version, crate::VERSION);
                }
            }
        }

        // Sort previous entries by session start time
        previous_bincodes.sort_by(|a, b| a.session_start.cmp(&b.session_start));

        // Load all previous session calculations and variables
        for bincode in previous_bincodes {
            // Merge previous variable declarations into the current session
            self.vartable.merge(&bincode.session_vartable)?;
            // Add previous calculation entry
            self.previous_entries.extend_from_slice(&bincode.entries);
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

    /// Add an entry to the current session.
    ///
    pub fn add_entry(&mut self, history_entry: &HistoryEntry) {
        self.entries.push(history_entry.clone());
    }

    /// Returns a concatination of all previous entries and all current entries.
    ///
    pub fn get_entries(&self) -> Vec<HistoryEntry> {
        let mut total_entries = self.previous_entries.clone();
        total_entries.extend_from_slice(&self.entries);

        total_entries
    }

    /// Gets an entry from the inverse index of the entry
    /// 
    pub fn get_entry_inv_index<'a>(&'a self, inverse_index: usize) -> Option<&'a HistoryEntry> {
        if inverse_index >= self.previous_entries.len() + self.entries.len() {
            return None
        }

        let inverse_index = inverse_index + 1; // Add 1 to the index, since the length is always 1 greater than the maximum index value

        let (entries, index) = match inverse_index.cmp(&self.entries.len()) {
            Ordering::Greater => {
                let index = &self.previous_entries.len() - (inverse_index - &self.entries.len());
                (&self.previous_entries, index)
            }
            _ => {
                let index = &self.entries.len() - inverse_index;
                (&self.entries, index)
            }
        };

        Some(&entries[index])
    }

    /// Gets an entry's inverse index from a given UUID
    /// 
    pub fn get_inv_index_from_uuid(&self, uuid: Uuid) -> Option<usize> {
        if let Some(index) = self.entries.iter().position(|entry| entry.entry_uuid == uuid) {
            return Some(self.entries.len() - index - 1) // Subtract 1 from the index, since the length is always 1 greater than the maximum index value
        }

        if let Some(index) = self.previous_entries.iter().position(|entry| entry.entry_uuid == uuid) {
            return Some(self.previous_entries.len() - (index - 1 - self.entries.len())) // Ditto
        }

        None
    }

    /// Gets an entry from a given UUID
    /// 
    pub fn get_entry_from_uuid<'a>(&'a self, uuid: Uuid) -> Option<&'a HistoryEntry> {
        if let Some(entry) = self.entries.iter().find(|entry| entry.entry_uuid == uuid) {
            return Some(entry)
        }

        if let Some(entry) = self.previous_entries.iter().find(|entry| entry.entry_uuid == uuid) {
            return Some(entry)
        }
        todo!()
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

    /// Create a history bincode from a session
    ///
    pub fn create_history_bincode(&self) -> HistoryBincode {
        HistoryBincode {
            version: self.version.clone(),
            session_start: self.session_start,
            session_uuid: self.session_uuid,
            decimal_places: self.decimal_places,
            session_vartable: self.vartable.clone(),
            entries: self.entries.clone(),
        }
    }

    /// Update the history file to reflect the current session
    ///
    pub fn update_file(&mut self) -> Result<(), Box<dyn Error>> {
        let history_bincode = self.create_history_bincode();

        let data = history_bincode.to_vec()?;

        // Create the file if it doesn't exist yet, clear it, and write the bincode
        let mut file = File::options()
            .read(false)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.history_file_path)?;

        file.write_all(&data)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use serial_test::*;

    const TWOPTWO: &str = "2 + 2";

    // Test the creation of a history manager
    #[test]
    #[serial]
    fn test_new_session() {
        // create a test session
        let mut session = Session::_new_test().unwrap();

        session.init().unwrap();

        // File should not exist yet!
        assert!(!&session.history_file_path.exists());

        // There should be no previous entries!
        assert_eq!(session.previous_entries.len(), 0);

        // There should also be no entries in the current bincode!
        assert_eq!(session.entries.len(), 0);

        session._test_purge().unwrap();
    }

    // Test adding entries to the entry manager
    #[test]
    #[serial]
    fn test_add_entry_session() {
        // create a test session
        let mut session = Session::_new_test().unwrap();

        session.init().unwrap();

        let expression = parser::parse_str(TWOPTWO, &mut session).unwrap();

        let history_entry = HistoryEntry::new(&expression, session.decimal_places);

        session.add_entry(&history_entry);

        // File should still not exist!
        assert!(!session.history_file_path.exists());

        // First entry should equal our expression!
        assert_eq!(session.get_entries()[0].to_string(), TWOPTWO);

        session._test_purge().unwrap();
    }

    // Test updating history files
    #[test]
    #[serial]
    fn test_update_file_session() {
        // create a test session
        let mut session = Session::_new_test().unwrap();

        session.init().unwrap();

        let expression = parser::parse_str(TWOPTWO, &mut session).unwrap();

        let history_entry = HistoryEntry::new(&expression, session.decimal_places);

        session.add_entry(&history_entry);

        session.update_file().unwrap();

        // File should now exist!
        assert!(&session.history_file_path.exists());

        // Check to make sure the bincode was written to correctly
        assert_eq!(
            &session.create_history_bincode(),
            &HistoryBincode::from_slice(&fs::read(&session.history_file_path).unwrap()).unwrap()
        );

        // Clean up!
        session._test_purge().unwrap();
    }

    // Test retrieving history from history files
    #[test]
    #[serial]
    fn test_retrive_session() {
        // create a test session
        let mut session1 = Session::_new_test().unwrap();

        session1.init().unwrap();

        let expression = parser::parse_str(TWOPTWO, &mut session1).unwrap();

        let history_entry = HistoryEntry::new(&expression, session1.decimal_places);

        session1.add_entry(&history_entry);

        session1.update_file().unwrap();

        let mut session2 = Session::_new_test().unwrap();

        session2.init().unwrap();

        // Make sure the previous entries of the second manager instance are equal to the current entries of the first manager instance
        assert_eq!(session2.previous_entries, session1.entries);

        session1._test_purge().unwrap();
    }

    // Test using the get by inverse index and get by uuid functions
    #[test]
    #[serial]
    fn test_retrieve_by_inv_index() {
        // create a test session
        let mut session = Session::_new_test().unwrap();

        session.init().unwrap();

        let expression = parser::parse_str(TWOPTWO, &mut session).unwrap();

        let history_entry = HistoryEntry::new(&expression, session.decimal_places);

        session.add_entry(&history_entry);

        let history_entry_2_inv_index = session.get_inv_index_from_uuid(history_entry.entry_uuid).unwrap();

        eprintln!("index = {}", history_entry_2_inv_index);

        let history_entry_2 = session.get_entry_inv_index(history_entry_2_inv_index).unwrap();

        assert_eq!(history_entry_2, &history_entry);

        session._test_purge().unwrap();
    }
}
