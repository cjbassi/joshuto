use joshuto::structs;
use joshuto::ui;

#[cfg(test)]
mod test;


#[derive(Clone, Debug)]
pub struct JoshutoPageState {
    pub start: usize,
    pub end: usize,
}

impl JoshutoPageState {
    pub fn new() -> Self
    {
        JoshutoPageState {
            start: 0,
            end: 0,
        }
    }

    pub fn update_page_state(&mut self, index: usize, win_rows: i32, vec_len: usize, offset: usize)
    {
        if self.end != win_rows as usize + self.start {
            self.end = self.start + win_rows as usize;
        }
        if self.end > vec_len {
            self.end = vec_len
        }

        if self.start + offset >= index {
            self.start = if index as usize <= offset {
                    0
                } else {
                    index as usize - offset
                };
            self.end = if self.start + win_rows as usize >= vec_len {
                    vec_len
                } else {
                    self.start + win_rows as usize
                };
            self.start = if self.end <= win_rows as usize {
                    0
                } else {
                    self.end - win_rows as usize
                };
        }
        if self.end <= index + offset {
            self.end = if index as usize + offset >= vec_len {
                    vec_len
                } else {
                    index as usize + offset
                };
            self.start = if self.end <= win_rows as usize {
                    0
                } else {
                    self.end - win_rows as usize
                };
            self.end = if self.start + win_rows as usize >= vec_len {
                    vec_len
                } else {
                    self.start + win_rows as usize
                };
        }
    }
}

#[derive(Debug, Clone)]
pub struct JoshutoPanel {
    pub win: ncurses::WINDOW,
    pub panel: ncurses::PANEL,
    pub rows: i32,
    pub cols: i32,
    /* coords (y, x) */
    pub coords: (usize, usize)
}

impl std::ops::Drop for JoshutoPanel {
    fn drop(&mut self)
    {
        ncurses::del_panel(self.panel);
        ncurses::delwin(self.win);
        ncurses::update_panels();
    }
}

impl JoshutoPanel {
    pub fn new(rows: i32, cols: i32, coords: (usize, usize)) -> Self
    {
        let win = ncurses::newwin(rows, cols, coords.0 as i32, coords.1 as i32);
        let panel = ncurses::new_panel(win);
        ncurses::leaveok(win, true);

        ncurses::wnoutrefresh(win);
        JoshutoPanel {
            win,
            panel,
            rows,
            cols,
            coords,
        }
    }

    pub fn move_to_top(&self)
    {
        ncurses::top_panel(self.panel);
    }
    pub fn move_to_bottom(&self)
    {
        ncurses::bottom_panel(self.panel);
    }
    pub fn queue_for_refresh(&self)
    {
        ncurses::wnoutrefresh(self.win);
    }

    pub fn display_contents(&self, dirlist: &mut structs::JoshutoDirList,
            scroll_offset: usize)
    {
        if self.non_empty_dir_checks(dirlist, scroll_offset) {
            Self::draw_dir_list(self, dirlist, ui::wprint_entry);
        }
    }

    pub fn display_contents_detailed(&self, dirlist: &mut structs::JoshutoDirList,
            scroll_offset: usize)
    {
        if self.non_empty_dir_checks(dirlist, scroll_offset) {
            Self::draw_dir_list(self, dirlist, ui::wprint_entry_detailed);
        }
    }

    pub fn draw_dir_list(win: &JoshutoPanel, dirlist: &structs::JoshutoDirList,
            draw_func: fn (&JoshutoPanel, &structs::JoshutoDirEntry, (usize, &str), (i32, i32)))
    {
        let dir_contents = &dirlist.contents;
        let (start, end) = (dirlist.pagestate.start, dirlist.pagestate.end);

        let curr_index = dirlist.index as usize;

        for i in start..end {
            let coord: (i32, i32) = (i as i32 - start as i32, 0);

            ncurses::wmove(win.win, coord.0, coord.1);
            let entry = &dir_contents[i];

            let mut attr: ncurses::attr_t = 0;
            if i == curr_index {
                attr = attr | ncurses::A_STANDOUT();
            }
            let attrs = ui::get_theme_attr(attr, entry);

            draw_func(win, entry, attrs.0, coord);

            ncurses::mvwchgat(win.win, coord.0, coord.1, -1, attrs.1, attrs.2);
        }
    }

    fn non_empty_dir_checks(&self, dirlist: &mut structs::JoshutoDirList, scroll_offset: usize) -> bool
    {
        if self.cols < 8 {
            return false;
        }
        let index = dirlist.index;
        let vec_len = dirlist.contents.len();
        if vec_len == 0 {
            ui::wprint_empty(self, "empty");
            return false;
        }
        ncurses::werase(self.win);

        if index >= 0 {
            dirlist.pagestate.update_page_state(index as usize, self.rows, vec_len, scroll_offset);
        }
        ncurses::wmove(self.win, 0, 0);
        return true;
    }
}

#[derive(Debug)]
pub struct JoshutoView {
    pub top_win: JoshutoPanel,
    pub tab_win: JoshutoPanel,
    pub left_win: JoshutoPanel,
    pub mid_win: JoshutoPanel,
    pub right_win: JoshutoPanel,
    pub bot_win: JoshutoPanel,
    pub win_ratio: (usize, usize, usize),
}

impl JoshutoView {
    pub fn new(win_ratio : (usize, usize, usize)) -> Self
    {
        let sum_ratio: usize = win_ratio.0 + win_ratio.1 + win_ratio.2;

        let mut term_rows: i32 = 0;
        let mut term_cols: i32 = 0;
        ncurses::getmaxyx(ncurses::stdscr(), &mut term_rows, &mut term_cols);
        let term_divide: i32 = term_cols / sum_ratio as i32;

        let win_xy: (i32, i32) = (1, term_cols - 5);
        let win_coord: (usize, usize) = (0, 0);
        let top_win = JoshutoPanel::new(win_xy.0, win_xy.1, win_coord);

        let win_xy: (i32, i32) = (1, 5);
        let win_coord: (usize, usize) = (0, term_cols as usize - 5);
        let tab_win = JoshutoPanel::new(win_xy.0, win_xy.1, win_coord);


        let win_xy: (i32, i32) = (term_rows - 2, (term_divide * win_ratio.0 as i32) - 1);
        let win_coord: (usize, usize) = (1, 0);
        let left_win = JoshutoPanel::new(win_xy.0, win_xy.1, win_coord);

        let win_xy: (i32, i32) = (term_rows - 2, (term_divide * win_ratio.1 as i32) - 1);
        let win_coord: (usize, usize) = (1, term_divide as usize * win_ratio.0);
        let mid_win = JoshutoPanel::new(win_xy.0, win_xy.1, win_coord);

        let win_xy: (i32, i32) = (term_rows - 2, (term_divide * win_ratio.2 as i32) - 1);
        let win_coord: (usize, usize) = (1, term_divide as usize * win_ratio.2);
        let right_win = JoshutoPanel::new(win_xy.0, win_xy.1, win_coord);


        let win_xy: (i32, i32) = (1, term_cols);
        let win_coord: (usize, usize) = (term_rows as usize - 1, 0);
        let bot_win = JoshutoPanel::new(win_xy.0, win_xy.1, win_coord);

        JoshutoView {
            top_win,
            tab_win,
            left_win,
            mid_win,
            right_win,
            bot_win,
            win_ratio,
        }
    }

    pub fn resize_views(&mut self)
    {
        let new_view = Self::new(self.win_ratio);

        self.top_win = new_view.top_win;
        self.bot_win = new_view.bot_win;
        self.tab_win = new_view.tab_win;
        self.left_win = new_view.left_win;
        self.mid_win = new_view.mid_win;
        self.right_win = new_view.right_win;
    }
}
