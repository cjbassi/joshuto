extern crate fs_extra;
extern crate ncurses;
extern crate open;

use std::env;
use std::path;

use commands::{self, JoshutoCommand, JoshutoRunnable};
use config::mimetype;
use context::JoshutoContext;
use preview;
use textfield::JoshutoTextField;
use ui;
use unix;
use window;

use mimetype_t;

#[derive(Clone, Debug)]
pub struct OpenFile;

impl OpenFile {
    pub fn new() -> Self {
        OpenFile
    }
    pub const fn command() -> &'static str {
        "open_file"
    }

    pub fn get_options<'a>(path: &path::PathBuf) -> Vec<&'a mimetype::JoshutoMimetypeEntry> {
        let mut mimetype_options: Vec<&mimetype::JoshutoMimetypeEntry> = Vec::new();

        match path.extension() {
            Some(file_ext) => {
                if let Some(file_ext) = file_ext.to_str() {
                    match mimetype_t.extension.get(file_ext) {
                        Some(s) => {
                            for option in s {
                                mimetype_options.push(&option);
                            }
                        }
                        None => {}
                    }
                }
            }
            None => {}
        }
        let detective = mime_detective::MimeDetective::new().unwrap();
        match detective.detect_filepath(path) {
            Ok(mime_type) => match mimetype_t.mimetype.get(mime_type.type_().as_str()) {
                Some(s) => {
                    for option in s {
                        mimetype_options.push(&option);
                    }
                }
                None => {}
            },
            Err(_) => {}
        }
        mimetype_options
    }

    fn into_directory(path: &path::PathBuf, context: &mut JoshutoContext) {
        let curr_tab = &mut context.tabs[context.curr_tab_index];

        match env::set_current_dir(path) {
            Ok(_) => {}
            Err(e) => {
                ui::wprint_err(
                    &context.views.window_bot,
                    format!("{}: {:?}", e, path).as_str(),
                );
                return;
            }
        }

        {
            let parent_list = curr_tab.parent_list.take();
            curr_tab.history.put_back(parent_list);

            let curr_list = curr_tab.curr_list.take();
            curr_tab.parent_list = curr_list;
        }

        curr_tab.curr_list = match curr_tab
            .history
            .pop_or_create(&path, &context.config_t.sort_type)
        {
            Ok(s) => Some(s),
            Err(e) => {
                ui::wprint_err(&context.views.left_win, e.to_string().as_str());
                None
            }
        };

        /* update curr_path */
        match path.strip_prefix(curr_tab.curr_path.as_path()) {
            Ok(s) => curr_tab.curr_path.push(s),
            Err(e) => {
                ui::wprint_err(&context.views.window_bot, e.to_string().as_str());
                return;
            }
        }
    }

    fn into_file(paths: &Vec<path::PathBuf>) {
        let mimetype_options = Self::get_options(&paths[0]);

        ncurses::savetty();
        ncurses::endwin();
        if mimetype_options.len() > 0 {
            unix::open_with_entry(paths, &mimetype_options[0]);
        } else {
            open::that(&paths[0]).unwrap();
        }
        ncurses::resetty();
        ncurses::refresh();
        ncurses::doupdate();
    }
}

impl JoshutoCommand for OpenFile {}

impl std::fmt::Display for OpenFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(Self::command())
    }
}

impl JoshutoRunnable for OpenFile {
    fn execute(&self, context: &mut JoshutoContext) {
        let mut path: Option<path::PathBuf> = None;
        if let Some(curr_list) = context.tabs[context.curr_tab_index].curr_list.as_ref() {
            if let Some(entry) = curr_list.get_curr_ref() {
                if entry.path.is_dir() {
                    path = Some(entry.path.clone());
                }
            }
        }
        if let Some(path) = path {
            Self::into_directory(&path, context);
            {
                let curr_tab = &mut context.tabs[context.curr_tab_index];
                curr_tab.refresh(
                    &context.views,
                    &context.config_t,
                    &context.username,
                    &context.hostname,
                );
            }
            preview::preview_file(context);
            ncurses::doupdate();
        } else {
            let paths: Option<Vec<path::PathBuf>> =
                match context.tabs[context.curr_tab_index].curr_list.as_ref() {
                    Some(s) => commands::collect_selected_paths(s),
                    None => None,
                };
            if let Some(paths) = paths {
                if paths.len() > 0 {
                    Self::into_file(&paths);
                } else {
                    ui::wprint_msg(&context.views.window_bot, "No files selected: 0");
                }
            } else {
                ui::wprint_msg(&context.views.window_bot, "No files selected: None");
            }
            ncurses::doupdate();
        }
    }
}

#[derive(Clone, Debug)]
pub struct OpenFileWith;

impl OpenFileWith {
    pub fn new() -> Self {
        OpenFileWith
    }
    pub const fn command() -> &'static str {
        "open_file_with"
    }

    pub fn open_with(paths: &Vec<path::PathBuf>) {
        const PROMPT: &str = ":open_with ";

        let mimetype_options: Vec<&mimetype::JoshutoMimetypeEntry> =
            OpenFile::get_options(&paths[0]);
        let user_input: Option<String>;
        {
            let (term_rows, term_cols) = ui::getmaxyx();

            let option_size = mimetype_options.len();
            let display_win = window::JoshutoPanel::new(
                option_size as i32 + 2,
                term_cols,
                (term_rows as u32 - option_size - 2, 0),
            );

            let mut display_vec: Vec<String> = Vec::with_capacity(option_size);
            for (i, val) in mimetype_options.iter().enumerate() {
                display_vec.push(format!("  {}\t{}", i, val));
            }
            display_vec.sort();

            display_win.move_to_top();
            ui::display_options(&display_win, &display_vec);
            ncurses::doupdate();

            let textfield =
                JoshutoTextField::new(1, term_cols, (term_rows as u32 - 1, 0), PROMPT.to_string());
            user_input = textfield.readline_with_initial("", "");
        }
        ncurses::doupdate();

        if let Some(user_input) = user_input {
            if user_input.len() == 0 {
                return;
            }
            match user_input.parse::<u32>() {
                Ok(s) => {
                    if s < mimetype_options.len() {
                        ncurses::savetty();
                        ncurses::endwin();
                        unix::open_with_entry(&paths, &mimetype_options[s]);
                        ncurses::resetty();
                        ncurses::refresh();
                    }
                }
                Err(_) => {
                    let args: Vec<String> = user_input
                        .split_whitespace()
                        .map(|x| String::from(x))
                        .collect();
                    ncurses::savetty();
                    ncurses::endwin();
                    unix::open_with_args(&paths, &args);
                    ncurses::resetty();
                    ncurses::refresh();
                }
            }
        }
    }
}

impl JoshutoCommand for OpenFileWith {}

impl std::fmt::Display for OpenFileWith {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(Self::command())
    }
}

impl JoshutoRunnable for OpenFileWith {
    fn execute(&self, context: &mut JoshutoContext) {
        if let Some(s) = context.tabs[context.curr_tab_index].curr_list.as_ref() {
            if let Some(paths) = commands::collect_selected_paths(s) {
                Self::open_with(&paths);
            }
        }
    }
}
