extern crate ncurses;

use structs;
use ui;
use utils::{Point, Rectangle};

#[derive(Debug, Clone)]
pub struct JoshutoPanel {
    pub window: ncurses::WINDOW,
    pub panel: ncurses::PANEL,
    pub rect: Rectangle,
}

impl std::ops::Drop for JoshutoPanel {
    fn drop(&mut self) {
        ncurses::del_panel(self.panel);
        ncurses::delwin(self.window);
        ncurses::update_panels();
    }
}

impl JoshutoPanel {
    pub fn new(rect: Rectangle) -> Self {
        let window = ncurses::newwin(
            rect.height() as i32,
            rect.width() as i32,
            rect.min.y,
            rect.min.x,
        );
        let panel = ncurses::new_panel(window);
        ncurses::leaveok(window, true);

        ncurses::wnoutrefresh(window);
        JoshutoPanel {
            window,
            panel,
            rect,
        }
    }

    pub fn move_to_top(&self) {
        ncurses::top_panel(self.panel);
    }
    pub fn queue_for_refresh(&self) {
        ncurses::wnoutrefresh(self.window);
    }

    pub fn display_contents(&self, dirlist: &mut structs::JoshutoDirList, scroll_offset: u32) {
        if self.non_empty_dir_checks(dirlist, scroll_offset) {
            Self::draw_dir_list(self, dirlist, ui::wprint_entry);
        }
    }

    pub fn display_contents_detailed(
        &self,
        dirlist: &mut structs::JoshutoDirList,
        scroll_offset: u32,
    ) {
        if self.non_empty_dir_checks(dirlist, scroll_offset) {
            Self::draw_dir_list(self, dirlist, ui::wprint_entry_detailed);
        }
    }

    pub fn draw_dir_list(
        win: &JoshutoPanel,
        dirlist: &structs::JoshutoDirList,
        draw_func: fn(&JoshutoPanel, &structs::JoshutoDirEntry, (u32, &str), Point),
    ) {
        let dir_contents = &dirlist.contents;
        let (start, end) = (dirlist.pagestate.start, dirlist.pagestate.end);

        let curr_index = dirlist.index as u32;

        for i in start..end {
            let point = Point::new(i as i32 - start as i32, 0);

            ncurses::wmove(win.window, point.y, point.x);
            let entry = &dir_contents[i];

            let mut attr: ncurses::attr_t = 0;
            if i == curr_index {
                attr = attr | ncurses::A_STANDOUT();
            }
            let attrs = ui::get_theme_attr(attr, entry);

            draw_func(win, entry, attrs.0, point);

            ncurses::mvwchgat(win.window, point.y, point.x, -1, attrs.1, attrs.2);
        }
    }

    fn non_empty_dir_checks(
        &self,
        dirlist: &mut structs::JoshutoDirList,
        scroll_offset: u32,
    ) -> bool {
        if self.rect.width() < 8 {
            return false;
        }
        let index = dirlist.index;
        let vec_len = dirlist.contents.len();
        if vec_len == 0 {
            ui::wprint_empty(self, "empty");
            return false;
        }
        ncurses::werase(self.window);

        if index >= 0 {
            dirlist.pagestate.update_page_state(
                index,
                self.rect.height() as i32,
                vec_len,
                scroll_offset,
            );
        }
        ncurses::wmove(self.window, 0, 0);
        return true;
    }
}
