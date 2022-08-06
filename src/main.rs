//! Entrypoint into the code!
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

use apecrunch::session::Session;
use apecrunch::tui::*;
use clap::Parser;
use std::io::Write;
use termcolor::Color;
use termcolor::ColorChoice;
use termcolor::ColorSpec;
use termcolor::StandardStream;
use termcolor::WriteColor;

// Command line arguments parsed through clap
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Print paths to the config and session/history files
    #[clap(short, long)]
    print_file_paths: bool,
}

/// Placeholder main function.
///
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

// Test general user things
#[cfg(test)]
mod tests {
    use apecrunch::op_engine;
    use apecrunch::parser;
    use apecrunch::variable::VarTable;

    #[test]
    fn test_two_plus_two() {
        let user_string = "2+2";
        let expected_result = "2 + 2 = 4";
        let mut vartable = VarTable::new();

        let tokens = parser::parse_str(user_string, &mut vartable).unwrap();

        let result = op_engine::get_equality(&tokens, &mut vartable, 0).unwrap();

        assert_eq!(result.to_string(0), expected_result);
    }

    #[test]
    fn test_order_of_ops() {
        let user_string = "1+2*3-4/-5^(6+7)";
        let expected_result = "1 + 2 * 3 - 4 / -5^( 6 + 7 ) = 7.0000000032768";
        let mut vartable = VarTable::new();

        let tokens = parser::parse_str(user_string, &mut vartable).unwrap();

        let result = op_engine::get_equality(&tokens, &mut vartable, 13).unwrap();

        assert_eq!(result.to_string(13), expected_result);
    }
}
