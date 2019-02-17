use std::path::PathBuf;

use config;
use history;
use sort;
use structs::JoshutoDirList;
use ui;
use window::{JoshutoPanel, JoshutoView};

use theme_t;

pub struct JoshutoTab {
    pub history: history::DirHistory,
    pub curr_path: PathBuf,
    pub parent_list: Option<JoshutoDirList>,
    pub curr_list: Option<JoshutoDirList>,
}

impl JoshutoTab {
    pub fn new(curr_path: PathBuf, sort_type: &sort::SortType) -> Result<Self, std::io::Error> {
        let mut history = history::DirHistory::new();
        history.populate_to_root(&curr_path, sort_type);

        let curr_list: JoshutoDirList = history.pop_or_create(&curr_path, sort_type)?;

        let parent_list: Option<JoshutoDirList> = match curr_path.parent() {
            Some(parent) => {
                let tmp_list = history.pop_or_create(&parent, sort_type)?;
                Some(tmp_list)
            }
            None => None,
        };

        let tab = JoshutoTab {
            curr_path,
            history,
            curr_list: Some(curr_list),
            parent_list,
        };
        Ok(tab)
    }

    pub fn reload_contents(&mut self, sort_type: &sort::SortType) {
        let mut list = self.curr_list.take();
        match list {
            Some(ref mut s) => {
                if s.path.exists() {
                    s.update_contents(sort_type).unwrap();
                }
            }
            None => {}
        };
        self.curr_list = list;

        list = self.parent_list.take();
        match list {
            Some(ref mut s) => {
                if s.path.exists() {
                    s.update_contents(sort_type).unwrap();
                }
            }
            None => {}
        };
        self.parent_list = list;
    }

    pub fn refresh(
        &mut self,
        views: &JoshutoView,
        config_t: &config::JoshutoConfig,
        username: &str,
        hostname: &str,
    ) {
        self.refresh_(
            views,
            config_t.tilde_in_titlebar,
            config_t.scroll_offset,
            username,
            hostname,
        );
    }

    pub fn refresh_(
        &mut self,
        views: &JoshutoView,
        tilde_in_titlebar: bool,
        scroll_offset: u32,
        username: &str,
        hostname: &str,
    ) {
        self.refresh_curr(&views.window_mid, scroll_offset);
        self.refresh_parent(&views.window_left, scroll_offset);
        self.refresh_path_status(&views.window_top, username, hostname, tilde_in_titlebar);
        self.refresh_file_status(&views.window_bot);
    }

    pub fn refresh_curr(&mut self, win: &JoshutoPanel, scroll_offset: u32) {
        if let Some(ref mut s) = self.curr_list {
            win.display_contents_detailed(s, scroll_offset);
            win.queue_for_refresh();
        }
    }

    pub fn refresh_parent(&mut self, win: &JoshutoPanel, scroll_offset: u32) {
        if let Some(ref mut s) = self.parent_list {
            win.display_contents(s, scroll_offset);
            win.queue_for_refresh();
        }
    }

    pub fn refresh_file_status(&self, win: &JoshutoPanel) {
        if let Some(ref dirlist) = self.curr_list {
            ncurses::werase(win.window);
            ncurses::wmove(win.window, 0, 0);

            if let Some(entry) = dirlist.get_curr_ref() {
                ui::wprint_file_mode(win.window, entry);
                ncurses::waddstr(win.window, " ");
                ncurses::waddstr(
                    win.window,
                    format!("{}/{} ", dirlist.index + 1, dirlist.contents.len()).as_str(),
                );
                ncurses::waddstr(win.window, "  ");
                ui::wprint_file_info(win.window, entry);
            }
            ncurses::wnoutrefresh(win.window);
        }
    }

    pub fn refresh_path_status(
        &self,
        win: &JoshutoPanel,
        username: &str,
        hostname: &str,
        tilde_in_titlebar: bool,
    ) {
        let path_str: &str = self.curr_path.to_str().unwrap();

        ncurses::werase(win.window);
        ncurses::wattron(win.window, ncurses::A_BOLD());
        ncurses::mvwaddstr(win.window, 0, 0, username);
        ncurses::waddstr(win.window, "@");
        ncurses::waddstr(win.window, hostname);

        ncurses::waddstr(win.window, " ");

        ncurses::wattron(win.window, ncurses::COLOR_PAIR(theme_t.directory.colorpair));
        if tilde_in_titlebar {
            let path_str = &path_str.replace(&format!("/home/{}", username), "~");
            ncurses::waddstr(win.window, path_str);
        } else {
            ncurses::waddstr(win.window, path_str);
        }
        ncurses::waddstr(win.window, "/");
        ncurses::wattroff(win.window, ncurses::COLOR_PAIR(theme_t.directory.colorpair));
        if let Some(ref dirlist) = self.curr_list {
            if let Some(entry) = dirlist.get_curr_ref() {
                ncurses::waddstr(win.window, &entry.file_name_as_string);
            }
        }
        ncurses::wattroff(win.window, ncurses::A_BOLD());
        win.queue_for_refresh();
    }
}
