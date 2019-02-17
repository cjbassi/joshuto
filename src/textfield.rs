extern crate ncurses;
extern crate unicode_width;

use config::keymap;
use utils::{Point, Rectangle};
use window::JoshutoPanel;

pub struct JoshutoTextField {
    pub window: JoshutoPanel,
    pub prompt: String,
}

impl JoshutoTextField {
    pub fn new(rect: Rectangle, prompt: String) -> Self {
        let window = JoshutoPanel::new(rect);
        ncurses::keypad(window.window, true);
        ncurses::scrollok(window.window, true);

        JoshutoTextField { window, prompt }
    }

    pub fn readline_with_initial(&self, prefix: &str, suffix: &str) -> Option<String> {
        let mut buf_vec: Vec<char> = Vec::with_capacity(prefix.len() + suffix.len());
        let mut curs_x: i32 = self.prompt.len() as i32;
        for ch in prefix.chars() {
            let char_len = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
            buf_vec.push(ch);
            curs_x = curs_x + char_len as i32;
        }
        let curr_index: u32 = buf_vec.len();

        for ch in suffix.chars() {
            buf_vec.push(ch);
        }
        ncurses::timeout(-1);
        let user_input = self.readline_(buf_vec, curs_x, curr_index);
        user_input
    }

    fn readline_(
        &self,
        mut buffer: Vec<(char)>,
        mut curs_x: i32,
        mut curr_index: u32,
    ) -> Option<String> {
        self.window.move_to_top();

        let prompt_len = self.prompt.len();
        let window = self.window.window;
        ncurses::wmove(window, self.window.rect.height() - 1, 0);
        ncurses::waddstr(window, &self.prompt);

        ncurses::doupdate();

        let point = Point::new(self.window.rect.min.x + prompt_len as i32, 0);

        loop {
            ncurses::wmove(window, point.y, point.x);
            {
                let str_ch: String = buffer.iter().collect();
                ncurses::waddstr(window, &str_ch);
            }
            ncurses::waddstr(window, "    ");

            ncurses::mvwchgat(window, point.y, curs_x, 1, ncurses::A_STANDOUT(), 0);
            ncurses::wrefresh(window);

            let ch = ncurses::wget_wch(window).unwrap();
            let ch = match ch {
                ncurses::WchResult::Char(s) => s as i32,
                ncurses::WchResult::KeyCode(s) => s,
            };

            if ch == keymap::ESCAPE {
                return None;
            } else if ch == keymap::ENTER {
                break;
            } else if ch == ncurses::KEY_HOME {
                if curr_index != 0 {
                    curs_x = point.x;
                    curr_index = 0;
                }
            } else if ch == ncurses::KEY_END {
                let buffer_len = buffer.len();
                if curr_index != buffer_len {
                    for i in curr_index..buffer_len {
                        curs_x = curs_x + buffer[i] as i32;
                    }
                    curr_index = buffer_len;
                }
            } else if ch == ncurses::KEY_LEFT {
                if curr_index > 0 {
                    curr_index = curr_index - 1;
                    curs_x = curs_x
                        - unicode_width::UnicodeWidthChar::width(buffer[curr_index]).unwrap_or(1)
                            as i32;
                }
            } else if ch == ncurses::KEY_RIGHT {
                let buffer_len = buffer.len();
                if curr_index < buffer_len {
                    curs_x = curs_x
                        + unicode_width::UnicodeWidthChar::width(buffer[curr_index]).unwrap_or(1)
                            as i32;
                    curr_index = curr_index + 1;
                }
            } else if ch == keymap::BACKSPACE {
                let buffer_len = buffer.len();
                if buffer_len == 0 {
                    continue;
                }

                if curr_index == buffer_len {
                    curr_index = curr_index - 1;
                    if let Some(ch) = buffer.pop() {
                        curs_x =
                            curs_x - unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1) as i32;
                    }
                } else if curr_index > 0 {
                    curr_index = curr_index - 1;
                    let ch = buffer.remove(curr_index);
                    curs_x =
                        curs_x - unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1) as i32;
                }
            } else if ch == ncurses::KEY_DC {
                let buffer_len = buffer.len();

                if buffer_len == 0 || curr_index == buffer_len {
                    continue;
                }

                if curr_index > 0 {
                    let ch = buffer.remove(curr_index);
                    if curr_index > buffer_len {
                        curr_index = curr_index - 1;
                        curs_x =
                            curs_x - unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1) as i32;
                    }
                } else if curr_index == 0 {
                    buffer.remove(curr_index);
                }
            } else {
                let ch = std::char::from_u32(ch as u32).unwrap();
                let char_len = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);

                buffer.insert(curr_index, ch);

                curs_x = curs_x + char_len as i32;
                curr_index = curr_index + 1;
            }
        }
        let user_str: String = buffer.iter().map(|ch| ch).collect();

        return Some(user_str);
    }
}
