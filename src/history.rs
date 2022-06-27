//! Plenty of useful structs and functions for saving and loading previous apecrunch sessions.
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
use crate::session::Session;
use bincode;
use lazy_static::*;
use regex::Regex;
use serde::Serialize;
use serde::*;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use uuid::Uuid;

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
    pub decimal_places: usize,
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
    pub fn new(expression: &Token, decimal_places: usize) -> Self {
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
    pub fn render_without_equality(&self, decimal_places: usize) -> String {
        if let Token::Equality(left, _right) = &self.expression {
            return left.to_string(decimal_places);
        }

        self.expression.to_string(decimal_places)
    }
}

/// Manages history entries, files, and etc.
///
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryManager {
    /// Path to the history file
    pub file_path: PathBuf,
    /// Previous entries found in previous sessions
    pub previous_entries: Vec<HistoryEntry>,
    /// Bincode of current session
    pub history_bincode: HistoryBincode,
}

impl HistoryManager {
    /// Creates a new history manager given the current session.
    ///
    pub fn new(session: &Session) -> Result<Self, Box<dyn Error>> {
        // Regex definitions for correctly identifying files
        lazy_static! {
            static ref HISTORY_FILE_RE: Regex =
                Regex::new(r"(.*)(history\-)(.+)(\.bincode\.lz4)").unwrap();
        }

        let mut previous_bincodes = Vec::<HistoryBincode>::new();

        // Go through each file in the session's data directory...
        for entry in fs::read_dir(&session.data_dir)? {
            let path = entry?.path().as_path().to_owned();

            let file_name = path.to_str().unwrap_or("");

            // And if the file name matches the regex...
            if HISTORY_FILE_RE.is_match(&file_name) {
                // Load it!
                let data = fs::read(path)?;
                let history_bincode = HistoryBincode::from_slice(&data)?;

                previous_bincodes.push(history_bincode);
            }
        }

        // Sort previous entries by session start time
        previous_bincodes.sort_by(|a, b| a.session_start.cmp(&b.session_start));

        let mut previous_entries = Vec::<HistoryEntry>::new();

        for session in previous_bincodes {
            previous_entries.extend_from_slice(&session.entries);
        }

        // Get the start time of the session, along with a fresh session uuid, the version of apecrunch, decimal places, etc. etc...
        let session_start = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let session_uuid = Uuid::new_v4();

        let version = crate::VERSION;
        let decimal_places = session.decimal_places;

        let entries = Vec::<HistoryEntry>::new();

        let mut file_path = session.data_dir.clone();

        file_path.push(format!("history-{}.bincode.lz4", session_uuid));

        // Should not happen, but in the 2^128 chance that it does...
        if file_path.exists() {
            panic!(
                "Random file name generation failed! File {} already exists!",
                file_path.to_str().unwrap()
            );
        }

        let history_bincode = HistoryBincode {
            version: version.to_string(),
            session_start: session_start,
            session_uuid: session_uuid,
            decimal_places: decimal_places,
            entries: entries,
        };

        Ok(Self {
            file_path: file_path,
            history_bincode: history_bincode,
            previous_entries: previous_entries,
        })
    }

    /// Returns a concatination of all previous entries and all current entries.
    ///
    pub fn get_entries(&self) -> Vec<HistoryEntry> {
        let mut total_entries = self.previous_entries.clone();
        total_entries.extend_from_slice(&self.history_bincode.entries);

        total_entries
    }

    /// Add an entry to the current session.
    ///
    pub fn add_entry(&mut self, history_entry: &HistoryEntry) {
        self.history_bincode.entries.push(history_entry.clone());
    }

    /// Update the history file to reflect the current session
    ///
    pub fn update_file(&mut self) -> Result<(), Box<dyn Error>> {
        let data = self.history_bincode.to_vec()?;

        // Create the file if it doesn't exist yet, clear it, and write the bincode
        let mut file = File::options()
            .read(false)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.file_path)?;

        file.write_all(&data)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use serial_test::*;

    const TWOPTWO: &str = "2 + 2";

    // Test the creation of a history manager
    #[test]
    #[serial]
    fn test_new_history_manager() {
        // create a test session
        let mut session = Session::_new_test();

        session.init().unwrap();

        let history_manager = HistoryManager::new(&session).unwrap();

        // File should not exist yet!
        assert!(!&history_manager.file_path.exists());

        // There should be no previous entries!
        assert_eq!(history_manager.previous_entries.len(), 0);

        // There should also be no entries in the current bincode!
        assert_eq!(history_manager.history_bincode.entries.len(), 0);

        session._test_purge().unwrap();
    }

    // Test adding entries to the entry manager
    #[test]
    #[serial]
    fn test_add_entry_history_manager() {
        // create a test session
        let mut session = Session::_new_test();

        session.init().unwrap();

        let mut history_manager = HistoryManager::new(&session).unwrap();

        let expression = parser::parse_str(TWOPTWO).unwrap();

        let history_entry = HistoryEntry::new(&expression, session.decimal_places);

        history_manager.add_entry(&history_entry);

        // File should still not exist!
        assert!(!history_manager.file_path.exists());

        // First entry should equal our expression!
        assert_eq!(history_manager.get_entries()[0].to_string(), TWOPTWO);

        session._test_purge().unwrap();
    }

    // Test updating history files
    #[test]
    #[serial]
    fn test_update_file_history_manager() {
        // create a test session
        let mut session = Session::_new_test();

        session.init().unwrap();

        let mut history_manager = HistoryManager::new(&session).unwrap();

        let expression = parser::parse_str(TWOPTWO).unwrap();

        let history_entry = HistoryEntry::new(&expression, session.decimal_places);

        history_manager.add_entry(&history_entry);

        history_manager.update_file().unwrap();

        // File should now exist!
        assert!(&history_manager.file_path.exists());

        // Check to make sure the bincode was written to correctly
        assert_eq!(
            history_manager.history_bincode.clone(),
            HistoryBincode::from_slice(&fs::read(history_manager.file_path).unwrap()).unwrap()
        );

        // Clean up!
        session._test_purge().unwrap();
    }

    // Test retrieving history from history files
    #[test]
    #[serial]
    fn test_retrive_history_files() {
        // create a test session
        let mut session = Session::_new_test();

        session.init().unwrap();

        let mut history_manager1 = HistoryManager::new(&session).unwrap();

        let expression = parser::parse_str(TWOPTWO).unwrap();

        let history_entry = HistoryEntry::new(&expression, session.decimal_places);

        history_manager1.add_entry(&history_entry);

        history_manager1.update_file().unwrap();

        let history_manager2 = HistoryManager::new(&session).unwrap();

        // Make sure the previous entries of the second manager instance are equal to the current entries of the first manager instance
        assert_eq!(
            history_manager2.previous_entries,
            history_manager1.history_bincode.entries
        );

        session._test_purge().unwrap();
    }
}
