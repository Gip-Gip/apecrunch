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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistoryJson {
    version: String,
    session_start: u64,
    session_uuid: Uuid,
    decimal_places: u32,
    entries: Vec<HistoryEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone)]
pub struct HistoryManager {
    file_name: String,
    previous_entries: Vec<HistoryEntry>,
    history_json: HistoryJson,
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
            if let Some(file_name) = entry?.path().to_str() {
                if HISTORY_FILE_RE.is_match(file_name) {
                    let data = lz4_flex::block::decompress_size_prepended(&fs::read(file_name)?)?;

                    let history_json: HistoryJson = serde_json::from_slice(&data)?;

                    previous_entries.extend_from_slice(&history_json.entries);
                }
            }
        }

        let session_start = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let session_uuid = Uuid::new_v4();

        let version = "0.0.1";
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
