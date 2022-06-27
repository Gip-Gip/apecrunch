//! Library bindings for the off chance you want to use one of these modules in your library
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

pub mod history;
pub mod number;
pub mod op_engine;
pub mod parser;
pub mod session;
pub mod tui;

/// Version of apecrunch, derived from the Cargo.toml version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
