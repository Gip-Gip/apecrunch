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

use cursive::views::LinearLayout;



use cursive::views::Dialog;
use cursive::views::SelectView;
use cursive::views::ScrollView;

use cursive::CursiveExt;
use cursive::Cursive;

use cursive::theme::*;
use cursive::views::EditView;
use cursive::view::Resizable;

use cursive::event::Event;

use cursive::align::HAlign;
use cursive::align::VAlign;
use crate::parser;
use crate::op_engine;


// Constants!

const TUI_ENTRYBAR_HEIGHT: usize = 1;

pub struct Tui
{
    cursive: Cursive,
}

impl Tui
{
    // tui::Tui::new() - create a new Tui instance
    //
    // DESCRIPTION:
    //  This constructor creates a new Cursive instance, initializes the Tui cache, and primes all event reactors.
    pub fn new() -> Self
    {
        let cursive = Cursive::new();

        let mut tui = Self
        {
            cursive: cursive,
        };

        let cache = TuiCache
        {
            mark_for_layout: true,
            entry_bar_cache: String::new(),
            history_cache: Vec::<String>::new(),
        };

        tui.cursive.set_user_data(cache);

        tui.prime();

        return tui;
    }

    // tui::Tui::run() - runs the Tui instance
    //
    // DESCRIPTION:
    //  Basically just a binding to cursive.run()
    pub fn run(&mut self)
    {
        self.cursive.run();
    }

    // tui::Tui::running() - execution loop
    //
    // ARGUMENTS:
    //  cursive: &mut Cursive - referenct to the Cursive instance
    // 
    // DESCRIPTION:
    //  Called 30 times a second and does basic mundane tasks which are non-reactive, like seeing if the program is running and properly initialized
    fn running(cursive: &mut Cursive)
    {
        let mut cache = match cursive.user_data::<TuiCache>()
        {
            Some(cache) =>
            {
                cache.clone()
            }
            None =>
            {
                panic!("Failed to initialize Cursive instance with cache! this should not happen!");
            }
        };

        if cache.mark_for_layout == true && cursive.is_running()
        {
            Self::layout(cursive);
            cache.mark_for_layout = false;
        }

        cursive.set_user_data(cache);
    }
    
    // tui::Tui::apply_theme_toml() - apply a theme from a TOML file
    //
    // ARGUMENTS:
    //  file_name: &str - the path to the TOML file
    //
    // DESCRIPTION:
    //  Loads a theme from the file specified and applies it to the current cursive instance
    pub fn apply_theme_toml(&mut self, file_name: &str)
    {
        let theme = load_theme_file(file_name).unwrap(); // Unwrap should be properly handled, will deal with later...

        self.cursive.set_theme(theme);
    }

    // tui::Tui::prime() - prime all the cursive events
    //
    // DESCRIPTION:
    //  Attach all the events to the cursive instance
    fn prime(&mut self)
    {
        // On a resize, mark the cursive instance for re-layout instead of calling layout directly. This combats a small bug I encountered where a resize event would trigger before the previous layout was finished
        self.cursive.set_on_pre_event
        (
            Event::WindowResize,
            |cursive|
            {
                let mut cache = match cursive.user_data::<TuiCache>()
                {
                    Some(cache) =>
                    {
                        cache.clone()
                    }
                    None =>
                    {
                        panic!("Failed to initialize Cursive instance with cache! this should not happen!");
                    }
                };

                cache.mark_for_layout = true;

                cursive.set_user_data(cache);
            }
        );

        // Prime the runtime loop
        self.cursive.set_on_pre_event(Event::Refresh, |cursive| Self::running(cursive));


        self.cursive.set_autorefresh(true); // Enable the refresh trigger
    }

    // tui::Tui::layout() - layout or re-layout the Tui
    //
    // DESCRIPTION:
    //  Grabs the (possibly new) width and height of the TUI and clears+lays out all of the views needed, preserving the content of all the views as well
    fn layout(cursive: &mut Cursive)
    {
        // Retrieve the cache
        let cache = match cursive.user_data::<TuiCache>()
        {
            Some(cache) =>
            {
                cache.clone()
            }
            None =>
            {
                panic!("Failed to initialize Cursive instance with cache! this should not happen!");
            }
        };

        // History view configuration!
        //

        let mut history_list = SelectView::new()
            .h_align(HAlign::Left)
            .v_align(VAlign::Top);
        
        for (i, entry) in cache.history_cache.iter().enumerate()
        {
            history_list.add_item(entry, i);
        }
        
        let mut history_scroll = ScrollView::new(history_list);

        history_scroll.scroll_to_important_area();

        let history = history_scroll.full_width().full_height();
        
        // Entry bar view configuration!
        //

        let entry_bar_fg = ColorType::Palette(PaletteColor::Primary);
        let entry_bar_bg = ColorType::Palette(PaletteColor::Highlight);

        let entry_bar_style = ColorStyle::new(entry_bar_bg, entry_bar_fg);

        let entry_bar = EditView::new()
            .style(entry_bar_style)
            .content(cache.entry_bar_cache)
            .on_edit(|cursive, text, cursor| Self::entry_bar_on_edit(cursive, text, cursor))
            .on_submit(|cursive, text| Self::entry_bar_on_submit(cursive, text))
            .full_width()
            .fixed_height(TUI_ENTRYBAR_HEIGHT);
        
        
        
        // Clear cursive and add all the views + reposition them all

        cursive.clear();
        let mut layout = LinearLayout::vertical()
            .child(history)
            .child(entry_bar);
        
        layout.set_focus_index(layout.len() - 1).unwrap();

        cursive.add_layer(layout);
    }

    // tui::Tui::entry_bar_on_edit - called each time the entry bar is edited
    //
    // ARGUMENTS:
    //  cursive: &mut Cursive - the cursive instance
    //  text: &str - the text stored in the entry bar
    //  _curser_pos: usize - the position of the cursor, currently unused...
    //
    // DESCRIPTION:
    //  A simple function that is called each time anything is typed or edited in the entry bar. Simply
    //  modifies the Tui cache to reflect the current entry bar data
    fn entry_bar_on_edit(cursive: &mut Cursive, text: &str, _cursor_pos: usize)
    {
        // Grab the cache
        let mut cache = match cursive.user_data::<TuiCache>()
        {
            Some(cache) =>
            {
                cache.clone()
            }
            None =>
            {
                panic!("Failed to initialize Cursive instance with cache! this should not happen!");
            }
        };

        cache.entry_bar_cache = text.to_string();

        cursive.set_user_data(cache); // Store the cache back with the updated entry_bar_cache
    }

    // tui::Tui::entry_bar_on_submit - called each time the entry bar is submitted
    //
    // ARGUMENTS:
    //  cursive: &mut Cursive - the cursive instance
    //  text: &str - the text stored in the entry bar
    //
    // DESCRIPTION:
    //  A simple function that is called each time the entry bar is submitted. Adds the current entry bar
    //  contents to the history cache and clears the entry bar.
    fn entry_bar_on_submit(cursive: &mut Cursive, text: &str)
    {
        // Grab the cache
        let mut cache = match cursive.user_data::<TuiCache>()
        {
            Some(cache) =>
            {
                cache.clone()
            }
            None =>
            {
                panic!("Failed to initialize Cursive instance with cache! this should not happen!");
            }
        };

        // Add the current entry bar contents to the history cache and clear the entry bar
        //

        // parse the text in the entry box
        let tokens = match parser::parse_str(text)
        {
            Ok(tokens) =>
            {
                tokens
            }
            Err(err) =>
            {
                let error_dialog = Dialog::text(format!("{}", err))
                    .title("Error!")
                    .button("Ok", |cursive| {cursive.pop_layer().unwrap();});

                cursive.add_layer(error_dialog);
                return;
            }
        };

        // Go through the tokens an operate on them, getting an equality
        let result = op_engine::get_equality(&tokens);

        cache.history_cache.push(result.to_string());
        cache.entry_bar_cache = String::new();

        cursive.set_user_data(cache); // Store the cache back with the updated history + entry bar cache

        Self::layout(cursive); // Update the layout
    }
}

#[derive(Debug, Clone)]
struct TuiCache
{
    pub mark_for_layout: bool,
    pub entry_bar_cache: String,
    pub history_cache: Vec<String>,
}

