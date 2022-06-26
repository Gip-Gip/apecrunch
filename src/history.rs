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
use crate::Session;
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HistoryBincode {
    version: String,
    session_start: u64,
    session_uuid: Uuid,
    decimal_places: u32,
    entries: Vec<HistoryEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HistoryEntry {
    entry_uuid: Uuid,
    expression: Token,
    rendition: String,
}

impl HistoryEntry {
    pub fn new(expression: &Token, decimal_places: usize) -> Self {
        let entry_uuid = Uuid::new_v4();
        return Self {
            entry_uuid: entry_uuid,
            expression: expression.clone(),
            rendition: expression.to_string(decimal_places),
        };
    }
    pub fn to_string(&self) -> String {
        return self.rendition.clone();
    }

    pub fn render_without_equality(&self, decimal_places: usize) -> String {
        if let Token::Equality(left, _right) = &self.expression {
            return left.to_string(decimal_places);
        }

        return self.expression.to_string(decimal_places);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HistoryManager {
    pub file_path: PathBuf,
    pub previous_entries: Vec<HistoryEntry>,
    pub history_bincode: HistoryBincode,
}

impl HistoryManager {
    pub fn new(session: &Session) -> Result<Self, Box<dyn Error>> {
        // Regex definitions for correctly identifying files
        lazy_static! {
            static ref HISTORY_FILE_RE: Regex =
                Regex::new(r"(.*)(history\-)(.+)(\.bincode\.lz4)").unwrap();
        }

        let mut previous_entries = Vec::<HistoryEntry>::new();

        for entry in fs::read_dir(&session.data_dir)? {
            let path = entry?.path().as_path().to_owned();

            let file_name = path.to_str().unwrap_or("");

            // Odd case where file entry would go through when file is freshly deleted, only encountered in testing
            // Still, check to make sure the file exists before trying to load it...
            if HISTORY_FILE_RE.is_match(&file_name) && path.exists() {
                let history_bincode: HistoryBincode = Self::bincode_from_file(path)?;

                previous_entries.extend_from_slice(&history_bincode.entries);
            }
        }

        let session_start = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let session_uuid = Uuid::new_v4();

        let version = crate::VERSION;
        let decimal_places = 12;

        let entries = Vec::<HistoryEntry>::new();

        let mut file_path = session.data_dir.clone();

        file_path.push(format!("history-{}.bincode.lz4", session_uuid));

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

        return Ok(Self {
            file_path: file_path,
            history_bincode: history_bincode,
            previous_entries: previous_entries,
        });
    }

    pub fn get_entries(&self) -> Vec<HistoryEntry> {
        let mut total_entries = self.previous_entries.clone();
        total_entries.extend_from_slice(&self.history_bincode.entries);
        return total_entries;
    }

    pub fn add_entry(&mut self, history_entry: &HistoryEntry) {
        self.history_bincode.entries.push(history_entry.clone());
    }

    pub fn bincode_from_file(path: PathBuf) -> Result<HistoryBincode, Box<dyn Error>> {
        let data = lz4_flex::block::decompress_size_prepended(&fs::read(path)?)?;

        return Ok(bincode::deserialize(&data)?);
    }

    pub fn update_file(&mut self) -> Result<(), Box<dyn Error>> {
        // Convert the history bincode struct into an lz4-compressed bincode stored in a vector
        let data =
            lz4_flex::block::compress_prepend_size(&bincode::serialize(&self.history_bincode)?);

        // Create the file if it doesn't exist yet, clear it, and write the bincode
        let mut file = File::options()
            .read(false)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.file_path)?;

        file.write_all(&data)?;

        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    const TWOPTWO: &str = "2 + 2";

    // Test the creation of a history manager
    #[test]
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

        session.purge().unwrap();
    }

    // Test adding entries to the entry manager
    #[test]
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

        session.purge().unwrap();
    }

    // Test updating history files
    #[test]
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
            HistoryManager::bincode_from_file(history_manager.file_path.clone()).unwrap()
        );

        // Clean up!
        session.purge().unwrap();
    }

    // Test retrieving history from history files
    #[test]
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

        session.purge().unwrap();
    }
}
