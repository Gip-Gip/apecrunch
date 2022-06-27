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

mod history;
mod number;
mod op_engine;
mod parser;
mod session;
mod tui;

use crate::session::Session;
use crate::tui::*;
use clap::Parser;
use std::io::Write;
use termcolor::Color;
use termcolor::ColorChoice;
use termcolor::ColorSpec;
use termcolor::StandardStream;
use termcolor::WriteColor;

/// Version of apecrunch, derived from the Cargo.toml version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Command-line arguments, parsed via CLAP.
/// 
/// **NOT PUBLIC.**
#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    print_file_paths: bool,
}

// Placeholder main function.
fn main() {
    let mut session = Session::new();

    session.init().unwrap();

    let args = Args::parse();
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    let mut green = ColorSpec::new();

    green.set_fg(Some(Color::Green));

    if args.print_file_paths {
        stdout.set_color(&green).unwrap();
        write!(&mut stdout, "\n\n\tConfig Directory:\t").unwrap();
        stdout.reset().unwrap();
        writeln!(&mut stdout, "{}", session.config_dir.to_str().unwrap()).unwrap();

        stdout.set_color(&green).unwrap();
        write!(&mut stdout, "\tHistory Directory:\t").unwrap();
        stdout.reset().unwrap();
        writeln!(&mut stdout, "{}\n\n", session.data_dir.to_str().unwrap()).unwrap();

        return;
    }

    stdout.reset().unwrap();

    let mut tui = Tui::new(session).unwrap();

    tui.run();
}
