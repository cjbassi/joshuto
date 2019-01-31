use std::path;
use std::sync;
use std::thread;

use joshuto::command;
use joshuto::config;
use joshuto::history;
use joshuto::sort;
use joshuto::structs::JoshutoDirList;
use joshuto::ui;
use joshuto::window::JoshutoView;
use joshuto::window::JoshutoPanel;

use joshuto::theme_t;

pub struct JoshutoContext {
    pub username: String,
    pub hostname: String,
    pub threads: Vec<(sync::mpsc::Receiver<command::ProgressInfo>, thread::JoinHandle<i32>)>,
    pub views: JoshutoView,
    pub curr_tab_index: usize,
    pub tabs: Vec<JoshutoTab>,

    pub config_t: config::JoshutoConfig,
}

impl<'a> JoshutoContext {
    pub fn new(config_t: config::JoshutoConfig) -> Self
    {
        let username: String = whoami::username();
        let hostname: String = whoami::hostname();

        let views: JoshutoView =
            JoshutoView::new(config_t.column_ratio);

        JoshutoContext {
            username,
            hostname,
            threads: Vec::new(),
            views,
            curr_tab_index: 0,
            tabs: Vec::new(),
            config_t,
        }
    }
    pub fn curr_tab_ref(&'a self) -> &'a JoshutoTab
    {
        &self.tabs[self.curr_tab_index]
    }
    pub fn curr_tab_mut(&'a mut self) -> &'a mut JoshutoTab
    {
        &mut self.tabs[self.curr_tab_index]
    }
}

pub struct JoshutoTab {
    pub history: history::DirHistory,
    pub curr_path: path::PathBuf,
    pub parent_list: Option<JoshutoDirList>,
    pub curr_list: Option<JoshutoDirList>,
}

impl JoshutoTab {
    pub fn new(curr_path: path::PathBuf, sort_type: &sort::SortType) -> Result<Self, std::io::Error>
    {
        let mut history = history::DirHistory::new();
        history.populate_to_root(&curr_path, sort_type);

        let curr_list: JoshutoDirList = history.pop_or_create(&curr_path, sort_type)?;

        let parent_list: Option<JoshutoDirList> =
            match curr_path.parent() {
                Some(parent) => {
                    let tmp_list = history.pop_or_create(&parent, sort_type)?;
                    Some(tmp_list)
                },
                None => { None },
            };

        let tab = JoshutoTab {
            curr_path,
            history,
            curr_list: Some(curr_list),
            parent_list,
        };
        Ok(tab)
    }

    pub fn reload_contents(&mut self, sort_type: &sort::SortType)
    {
        let mut list = self.curr_list.take();
        match list {
            Some(ref mut s) => {
                if s.path.exists() {
                    s.update_contents(sort_type).unwrap();
                }
            },
            None => {},
        };
        self.curr_list = list;

        list = self.parent_list.take();
        match list {
            Some(ref mut s) => {
                if s.path.exists() {
                    s.update_contents(sort_type).unwrap();
                }
            },
            None => {},
        };
        self.parent_list = list;
    }

    pub fn refresh(&mut self, views: &JoshutoView, config_t: &config::JoshutoConfig,
            username: &str, hostname: &str)
    {
        self.refresh_(views, config_t.scroll_offset, username, hostname);
    }

    pub fn refresh_(&mut self, views: &JoshutoView, scroll_offset: usize,
            username: &str, hostname: &str)
    {
        self.refresh_curr(&views.mid_win, scroll_offset);
        self.refresh_parent(&views.left_win, scroll_offset);
        self.refresh_path_status(&views.top_win, username, hostname);
        self.refresh_file_status(&views.bot_win);
    }

    pub fn refresh_curr(&mut self, win: &JoshutoPanel, scroll_offset: usize)
    {
        if let Some(ref mut s) = self.curr_list {
            win.display_contents_detailed(s, scroll_offset);
            win.queue_for_refresh();
        }
    }

    pub fn refresh_parent(&mut self, win: &JoshutoPanel, scroll_offset: usize)
    {
        if let Some(ref mut s) = self.parent_list {
            win.display_contents(s, scroll_offset);
            win.queue_for_refresh();
        }
    }

    pub fn refresh_file_status(&self, win: &JoshutoPanel)
    {
        if let Some(ref dirlist) = self.curr_list {
            ncurses::werase(win.win);
            ncurses::wmove(win.win, 0, 0);

            if let Some(entry) = dirlist.get_curr_ref() {
                ui::wprint_file_mode(win.win, entry);
                ncurses::waddstr(win.win, " ");
                ncurses::waddstr(win.win, format!("{}/{} ", dirlist.index + 1, dirlist.contents.len()).as_str());
                ncurses::waddstr(win.win, "  ");
                ui::wprint_file_info(win.win, entry);
            }
            ncurses::wnoutrefresh(win.win);
        }
    }

    pub fn refresh_path_status(&self, win: &JoshutoPanel, username: &str, hostname: &str)
    {
        let path_str: &str = self.curr_path.to_str().unwrap();

        ncurses::werase(win.win);
        ncurses::wattron(win.win, ncurses::A_BOLD());
        ncurses::mvwaddstr(win.win, 0, 0, username);
        ncurses::waddstr(win.win, "@");
        ncurses::waddstr(win.win, hostname);

        ncurses::waddstr(win.win, " ");

        ncurses::wattron(win.win, ncurses::COLOR_PAIR(theme_t.directory.colorpair));
        ncurses::waddstr(win.win, path_str);
        ncurses::waddstr(win.win, "/");
        ncurses::wattroff(win.win, ncurses::COLOR_PAIR(theme_t.directory.colorpair));
        if let Some(ref dirlist) = self.curr_list {
            if let Some(entry) = dirlist.get_curr_ref() {
                ncurses::waddstr(win.win, &entry.file_name_as_string);
            }
        }
        ncurses::wattroff(win.win, ncurses::A_BOLD());
        win.queue_for_refresh();
    }
}
