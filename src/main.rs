mod tui;
mod parser;
mod number;
mod op_engine;

use crate::tui::*;
// Placeholder main function
fn main() {
    let mut tui = Tui::new();

    tui.apply_theme_toml("etc/theme.toml");

    tui.run();
}