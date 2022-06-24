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
use lazy_static::*;
use regex::Regex;
use serde::Serialize;
use serde::*;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HistoryJson {
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
    pub fn new(expression: &Token) -> Self {
        let entry_uuid = Uuid::new_v4();
        return Self {
            entry_uuid: entry_uuid,
            expression: expression.clone(),
            rendition: expression.to_string(),
        };
    }
    pub fn to_string(&self) -> String {
        return self.rendition.clone();
    }

    pub fn render_without_equality(&self) -> String {
        if let Token::Equality(left, _right) = &self.expression {
            return left.to_string();
        }

        return self.expression.to_string();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HistoryManager {
    pub file_name: String,
    pub previous_entries: Vec<HistoryEntry>,
    pub history_json: HistoryJson,
}

impl HistoryManager {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Regex definitions for correctly identifying files
        lazy_static! {
            static ref HISTORY_FILE_RE: Regex =
                Regex::new(r"(.*)(history\-)(.+)(\.json\.lz4)").unwrap();
        }

        let mut previous_entries = Vec::<HistoryEntry>::new();

        for entry in fs::read_dir("etc/")? {
            let path = entry?.path().as_path().to_owned();

            let file_name = path.to_str().unwrap_or("");

            // Odd case where file entry would go through when file is freshly deleted, only encountered in testing
            // Still, check to make sure the file exists before trying to load it...
            if HISTORY_FILE_RE.is_match(&file_name) && path.exists() {
                let history_json: HistoryJson = Self::json_from_file(&file_name)?;

                previous_entries.extend_from_slice(&history_json.entries);
            }
        }

        let session_start = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let session_uuid = Uuid::new_v4();

        let version = crate::VERSION;
        let decimal_places = 12;

        let entries = Vec::<HistoryEntry>::new();

        let file_name = format!("etc/history-{}.json.lz4", session_uuid);

        if Path::new(&file_name).exists() {
            panic!(
                "Random file name generation failed! File {} already exists!",
                file_name
            );
        }

        let history_json = HistoryJson {
            version: version.to_string(),
            session_start: session_start,
            session_uuid: session_uuid,
            decimal_places: decimal_places,
            entries: entries,
        };

        return Ok(Self {
            file_name: file_name,
            history_json: history_json,
            previous_entries: previous_entries,
        });
    }

    pub fn get_entries(&self) -> Vec<HistoryEntry> {
        let mut total_entries = self.previous_entries.clone();
        total_entries.extend_from_slice(&self.history_json.entries);
        return total_entries;
    }

    pub fn add_entry(&mut self, history_entry: &HistoryEntry) {
        self.history_json.entries.push(history_entry.clone());
    }

    pub fn json_from_file(string: &str) -> Result<HistoryJson, Box<dyn Error>> {
        let data = lz4_flex::block::decompress_size_prepended(&fs::read(string)?)?;

        return Ok(serde_json::from_slice(&data)?);
    }

    pub fn update_file(&mut self) -> Result<(), Box<dyn Error>> {
        // Convert the history json struct into an lz4-compressed json stored in a vector
        let data = lz4_flex::block::compress_prepend_size(&serde_json::to_vec(&self.history_json)?);

        // Create the file if it doesn't exist yet, clear it, and write the json
        let mut file = File::options()
            .read(false)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.file_name)?;

        file.write_all(&data)?;

        return Ok(());
    }

    pub fn delete_file(&self) -> Result<(), Box<dyn Error>> {
        fs::remove_file(&self.file_name)?;
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
        let history_manager = HistoryManager::new().unwrap();

        // File should not exist yet!
        assert!(!Path::new(&history_manager.file_name).exists());

        // There should be no previous entries!
        assert_eq!(history_manager.previous_entries.len(), 0);

        // There should also be no entries in the current json!
        assert_eq!(history_manager.history_json.entries.len(), 0);
    }

    // Test adding entries to the entry manager
    #[test]
    fn test_add_entry_history_manager() {
        let mut history_manager = HistoryManager::new().unwrap();

        let expression = parser::parse_str(TWOPTWO).unwrap();

        let history_entry = HistoryEntry::new(&expression);

        history_manager.add_entry(&history_entry);

        // File should still not exist!
        assert!(!Path::new(&history_manager.file_name).exists());

        // First entry should equal our expression!
        assert_eq!(history_manager.get_entries()[0].to_string(), TWOPTWO);
    }

    // Test updating history files
    #[test]
    fn test_update_file_history_manager() {
        let mut history_manager = HistoryManager::new().unwrap();

        let expression = parser::parse_str(TWOPTWO).unwrap();

        let history_entry = HistoryEntry::new(&expression);

        history_manager.add_entry(&history_entry);

        history_manager.update_file().unwrap();

        // File should now exist!
        assert!(Path::new(&history_manager.file_name).exists());

        // Check to make sure the json was written to correctly
        assert_eq!(
            history_manager.history_json,
            HistoryManager::json_from_file(history_manager.file_name.as_str()).unwrap()
        );

        // Clean up!
        history_manager.delete_file().unwrap();
    }

    // Test retrieving history from history files
    #[test]
    fn test_retrive_history_files() {
        let mut history_manager1 = HistoryManager::new().unwrap();

        let expression = parser::parse_str(TWOPTWO).unwrap();

        let history_entry = HistoryEntry::new(&expression);

        history_manager1.add_entry(&history_entry);

        history_manager1.update_file().unwrap();

        let history_manager2 = HistoryManager::new().unwrap();

        // Make sure the previous entries of the second manager instance are equal to the current entries of the first manager instance
        assert_eq!(
            history_manager2.previous_entries,
            history_manager1.history_json.entries
        );

        // Clean up!
        history_manager1.delete_file().unwrap();
    }
}
