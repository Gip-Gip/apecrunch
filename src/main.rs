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
    let mut session = Session::new().unwrap();

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
    use apecrunch::session::Session;
use apecrunch::op_engine;
    use apecrunch::parser;

    #[test]
    fn test_two_plus_two() {
        let user_string = "2+2";
        let expected_result = "2 + 2 = 4";
        let mut session = Session::_new_test().unwrap();

        let tokens = parser::parse_str(user_string, &mut session).unwrap();

        session.decimal_places = 0;

        let result = op_engine::get_equality(&tokens, &mut session).unwrap();

        assert_eq!(result.to_string(&session), expected_result);
    }

    #[test]
    fn test_order_of_ops() {
        let user_string = "1+2*3-4/-5^(6+7)";
        let expected_result = "1 + 2 * 3 - 4 / -5^( 6 + 7 ) = 7.0000000032768";
        let mut session = Session::_new_test().unwrap();

        let tokens = parser::parse_str(user_string, &mut session).unwrap();

        session.decimal_places = 13;

        let result = op_engine::get_equality(&tokens, &mut session).unwrap();

        assert_eq!(result.to_string(&session), expected_result);
    }

    #[test]
    fn test_roots() {
        let user_string1 = "16^0.5";
        let expected_result1 = "16^0.5 = 4";
        let user_string2 = "999999999^(2/3)";
        let expected_result2 = "999999999^( 2 / 3 ) = 999999.999333333333222222222172839506...";
        let mut session = Session::_new_test().unwrap();

        let tokens1 = parser::parse_str(user_string1, &mut session).unwrap();

        session.decimal_places = 1;

        let result1 = op_engine::get_equality(&tokens1, &mut session).unwrap();

        let tokens2 = parser::parse_str(user_string2, &mut session).unwrap();

        session.decimal_places = 30;

        let result2 = op_engine::get_equality(&tokens2, &mut session).unwrap();

        assert_eq!(result1.to_string(&session), expected_result1);
        assert_eq!(result2.to_string(&session), expected_result2);
    }
}
