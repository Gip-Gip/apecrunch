//! Provides abstraction for setting up and running the TUI.
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

use crate::history::HistoryEntry;
use crate::history::HistoryManager;
use crate::session::Session;
use cursive::view::Nameable;
use cursive::view::Selector;
use cursive::views::LinearLayout;
use cursive::views::ViewRef;
use cursive::View;
use std::error::Error;

use cursive::views::Dialog;
use cursive::views::ScrollView;
use cursive::views::SelectView;

use cursive::Cursive;
use cursive::CursiveExt;

use cursive::theme::*;
use cursive::view::Resizable;
use cursive::views::EditView;

use cursive::event::Event;
use cursive::event::Key;

use crate::op_engine;
use crate::parser;
use cursive::align::HAlign;
use cursive::align::VAlign;

// Constants!

/// Height of the entry bar.
const TUI_ENTRYBAR_HEIGHT: usize = 1;

/// ID of the entry bar view.
const TUI_ENTRYBAR_ID: &str = "entry_bar";
/// ID of the history view.
const TUI_HISTORY_ID: &str = "history";
/// ID of the layout view.
const TUI_LAYOUT_ID: &str = "layout";

/// Tui abstraction struct.
///
/// Currently uses the Cursive crate.
///
pub struct Tui {
    cursive: Cursive,
}

impl Tui {
    /// Creates a new Tui instance with a given session.
    ///
    /// Returns an error if there is one.
    ///
    pub fn new(session: Session) -> Result<Self, Box<dyn Error>> {
        // Create a new Cursive instance.
        let cursive = Cursive::new();

        let mut tui = Self { cursive: cursive };

        tui.apply_theme_toml(session.get_theme_file_path().to_str().unwrap());

        let cache = TuiCache {
            entry_bar_cursor_pos: 0,
            history_manager: HistoryManager::new(&session)?,
            session: session,
        };

        tui.cursive.set_user_data(cache);

        tui.prime(); // Prime all event handlers.
        tui.layout(); // Lay out all of the views.

        Ok(tui)
    }

    /// Run the cursive instance.
    ///
    pub fn run(&mut self) {
        self.cursive.run_crossterm().unwrap();
    }

    /// Get a theme from the given toml file and apply it.
    ///
    pub fn apply_theme_toml(&mut self, file_name: &str) {
        let theme = load_theme_file(file_name).unwrap(); // Unwrap should be properly handled, will deal with later...

        self.cursive.set_theme(theme);
    }

    /// Prime all event handlers.
    ///
    /// **NOT PUBLIC.**
    ///
    fn prime(&mut self) {
        // Bind the escape key to cursive.quit()
        self.cursive
            .set_on_pre_event(Event::Key(Key::Esc), |cursive| cursive.quit());
    }

    /// Lay out all of the views.
    ///
    /// **NOT PUBLIC.**
    ///
    fn layout(&mut self) {
        // Grab the cache.
        let cache = match self.cursive.user_data::<TuiCache>() {
            Some(cache) => cache.clone(),
            None => {
                panic!("Failed to initialize Cursive instance with cache! this should not happen!");
            }
        };

        // History view configuration!
        //

        let mut history_list = SelectView::new()
            .on_submit(|cursive, index| Self::history_on_select(cursive, *index))
            .h_align(HAlign::Left)
            .v_align(VAlign::Top);

        for (i, entry) in cache.history_manager.get_entries().iter().enumerate() {
            history_list.add_item(&entry.to_string(), i);
        }

        let mut history_scroll = ScrollView::new(history_list.with_name(TUI_HISTORY_ID));

        history_scroll.scroll_to_important_area();

        let history = history_scroll.full_width().full_height();

        // Entry bar view configuration!
        //

        let entry_bar_fg = ColorType::Palette(PaletteColor::Primary);
        let entry_bar_bg = ColorType::Palette(PaletteColor::Highlight);

        let entry_bar_style = ColorStyle::new(entry_bar_bg, entry_bar_fg);

        let entry_bar = EditView::new()
            .style(entry_bar_style)
            .on_edit(|cursive, text, cursor| Self::entry_bar_on_edit(cursive, text, cursor))
            .on_submit(|cursive, text| Self::entry_bar_on_submit(cursive, text));

        let entry_bar = entry_bar
            .with_name(TUI_ENTRYBAR_ID)
            .full_width()
            .fixed_height(TUI_ENTRYBAR_HEIGHT);

        // Clear cursive and add all the views + reposition them all

        self.cursive.clear();
        let mut layout = LinearLayout::vertical().child(history).child(entry_bar);

        // Focus on the entry bar
        layout.focus_view(&Selector::Name(TUI_ENTRYBAR_ID)).unwrap();

        // Make sure there isn't an unnessicary border and make the layout fullscreen
        self.cursive
            .add_fullscreen_layer(layout.with_name(TUI_LAYOUT_ID));
    }

    /// Called each time the entry bar is edited.
    ///
    /// **NOT PUBLIC.**
    fn entry_bar_on_edit(cursive: &mut Cursive, _text: &str, cursor_pos: usize) {
        // Grab the cache
        let mut cache = match cursive.user_data::<TuiCache>() {
            Some(cache) => cache.clone(),
            None => {
                panic!("Failed to initialize Cursive instance with cache! this should not happen!");
            }
        };

        // For some reason you cannot grab the cursor position from entrybar views, only set it. We must grab it from this function and use it later if necissary...
        cache.entry_bar_cursor_pos = cursor_pos;

        cursive.set_user_data(cache); // Store the cache back with the updated entry_bar_cache.
    }

    /// Called each time an entry bar is submitted(the user presses enter).
    ///
    ///
    fn entry_bar_on_submit(cursive: &mut Cursive, text: &str) {
        // Grab the cache
        let mut cache = match cursive.user_data::<TuiCache>() {
            Some(cache) => cache.clone(),
            None => {
                panic!("Failed to initialize Cursive instance with cache! this should not happen!");
            }
        };

        let mut entry_bar: ViewRef<EditView> = cursive.find_name(TUI_ENTRYBAR_ID).unwrap();
        let mut history: ViewRef<SelectView<usize>> = cursive.find_name(TUI_HISTORY_ID).unwrap();

        // Add the current entry bar contents to the history cache and clear the entry bar.
        //

        // parse the text in the entry box
        let tokens = match parser::parse_str(text, &mut cache.session.vartable) {
            Ok(tokens) => tokens,
            Err(error) => {
                Self::nonfatal_error_dialog(cursive, error);
                return;
            }
        };

        // Go through the tokens an operate on them, getting an equality.
        let result = op_engine::get_equality(&tokens);
        let entry = &HistoryEntry::new(&result, cache.session.decimal_places);
        let index = cache.history_manager.get_entries().len();

        cache.history_manager.add_entry(&entry);
        history.add_item(entry.to_string(), index);

        if let Result::Err(error) = cache.history_manager.update_file() {
            Self::nonfatal_error_dialog(cursive, error);
            return;
        }

        cache.entry_bar_cursor_pos = 0;

        entry_bar.set_content(""); // Ignore callback.

        cursive.set_user_data(cache); // Store the cache back with the updated history + entry bar cache.
    }

    /// Called when an entry in the history list is selected.
    ///
    /// Takes the selected entry and inserts it at the cursor position.
    ///
    /// **NOT PUBLIC.**
    ///
    fn history_on_select(cursive: &mut Cursive, index: usize) {
        // Grab the cache
        let cache = match cursive.user_data::<TuiCache>() {
            Some(cache) => cache.clone(),
            None => {
                panic!("Failed to initialize Cursive instance with cache! this should not happen!");
            }
        };

        let mut entry_bar: ViewRef<EditView> = cursive.find_name(TUI_ENTRYBAR_ID).unwrap();
        let mut layout: ViewRef<LinearLayout> = cursive.find_name(TUI_LAYOUT_ID).unwrap();

        let entry_bar_content = entry_bar.get_content();

        let mut curser_pos = cache.entry_bar_cursor_pos;

        // Get the selected history entry.
        let entry = &cache.history_manager.get_entries()[index]
            .render_without_equality(cache.session.decimal_places);

        // Insert the selected history entry into the entry bar at the cursor position.
        let left = &entry_bar_content[..curser_pos];
        let right = &entry_bar_content[curser_pos..];

        let entry_bar_content = format!("{}{}{}", left, entry, right);

        curser_pos += entry_bar_content.len();

        entry_bar.set_content(entry_bar_content);
        entry_bar.set_cursor(curser_pos);

        // Return focus to the entry bar.
        layout.focus_view(&Selector::Name(TUI_ENTRYBAR_ID)).unwrap();
    }

    /// Create a dialog for non fatal errors.
    ///
    pub fn nonfatal_error_dialog(cursive: &mut Cursive, error: Box<dyn Error>) {
        let error_dialog =
            Dialog::text(format!("{}", error))
                .title("Error!")
                .button("Ok", |cursive| {
                    cursive.pop_layer().unwrap();
                });

        cursive.add_layer(error_dialog);
    }
}

/// Cache for everything that needs to be retained during the cursive session
///
/// **NOT PUBLIC.**
///
#[derive(Debug, Clone)]
struct TuiCache {
    pub entry_bar_cursor_pos: usize,
    pub history_manager: HistoryManager,
    pub session: Session,
}
