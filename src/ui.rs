extern crate libc;
extern crate ncurses;
extern crate unicode_width;

use std::fs;
use std::time;

use config::JoshutoColorTheme;
use context::JoshutoContext;
use structs;
use unix;
use utils::Point;
use window;

use theme_t;

pub const ERR_COLOR: i16 = 240;
pub const EMPTY_COLOR: i16 = 241;

pub fn init_ncurses() {
    ncurses::setlocale(ncurses::LcCategory::all, "");

    ncurses::initscr();
    ncurses::cbreak();

    ncurses::keypad(ncurses::stdscr(), true);
    ncurses::start_color();
    ncurses::use_default_colors();
    ncurses::noecho();
    ncurses::set_escdelay(0);

    process_theme();

    ncurses::addstr("Loading...");
    ncurses::curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    ncurses::refresh();
}

fn process_theme() {
    for pair in theme_t.colorpair.iter() {
        ncurses::init_pair(pair.id, pair.fg, pair.bg);
    }

    /* error message */
    ncurses::init_pair(ERR_COLOR, ncurses::COLOR_RED, -1);
    /* empty */
    ncurses::init_pair(EMPTY_COLOR, ncurses::COLOR_WHITE, ncurses::COLOR_RED);
}

pub fn end_ncurses() {
    ncurses::endwin();
}

pub fn get_terminal_dimensions() -> (u32, u32) {
    let mut width: i32 = 0;
    let mut height: i32 = 0;
    ncurses::getmaxyx(ncurses::stdscr(), &mut height, &mut width);
    (width as u32, height as u32)
}

pub fn display_options(win: &window::JoshutoPanel, vals: &Vec<String>) {
    ncurses::werase(win.window);
    ncurses::mvwhline(win.window, 0, 0, 0, win.rect.width() as i32);

    for (i, val) in vals.iter().enumerate() {
        ncurses::wmove(win.window, (i + 1) as i32, 0);
        ncurses::waddstr(win.window, val.as_str());
    }
    ncurses::wnoutrefresh(win.window);
}

pub fn wprint_msg(win: &window::JoshutoPanel, msg: &str) {
    ncurses::werase(win.window);
    ncurses::mvwaddstr(win.window, 0, 0, msg);
    ncurses::wnoutrefresh(win.window);
}

pub fn wprint_err(win: &window::JoshutoPanel, msg: &str) {
    ncurses::werase(win.window);
    ncurses::wattron(win.window, ncurses::A_BOLD());
    ncurses::wattron(win.window, ncurses::COLOR_PAIR(ERR_COLOR));
    ncurses::mvwaddstr(win.window, 0, 0, msg);
    ncurses::wattroff(win.window, ncurses::COLOR_PAIR(ERR_COLOR));
    ncurses::wattroff(win.window, ncurses::A_BOLD());
    ncurses::wnoutrefresh(win.window);
}

pub fn wprint_empty(win: &window::JoshutoPanel, msg: &str) {
    ncurses::werase(win.window);
    ncurses::wattron(win.window, ncurses::COLOR_PAIR(EMPTY_COLOR));
    ncurses::mvwaddstr(win.window, 0, 0, msg);
    ncurses::wattroff(win.window, ncurses::COLOR_PAIR(EMPTY_COLOR));
    ncurses::wnoutrefresh(win.window);
}

fn wprint_file_name(win: ncurses::WINDOW, file_name: &str, point: Point, mut space_avail: u32) {
    let name_visual_space = unicode_width::UnicodeWidthStr::width(file_name);
    if name_visual_space < space_avail {
        ncurses::mvwaddstr(win, point.y, point.x, &file_name);
        return;
    }
    if let Some(ext) = file_name.rfind('.') {
        let extension: &str = &file_name[ext..];
        let ext_len = unicode_width::UnicodeWidthStr::width(extension);
        if space_avail > ext_len {
            space_avail = space_avail - ext_len;
            ncurses::mvwaddstr(win, point.y, space_avail as i32, &extension);
        }
    }
    if space_avail > 2 {
        space_avail = space_avail - 2;
    }

    ncurses::wmove(win, point.y, point.x);

    let mut trim_index: u32 = file_name.len();

    let mut total: u32 = 0;
    for (index, ch) in file_name.char_indices() {
        if total >= space_avail {
            trim_index = index;
            break;
        }
        total = total + unicode_width::UnicodeWidthChar::width(ch).unwrap_or(2);
    }
    ncurses::waddstr(win, &file_name[..trim_index]);
    ncurses::waddstr(win, "…");
}

pub fn wprint_entry(
    win: &window::JoshutoPanel,
    file: &structs::JoshutoDirEntry,
    prefix: (u32, &str),
    point: Point,
) {
    ncurses::waddstr(win.window, prefix.1);
    let space_avail: u32;
    if win.rect.width() >= prefix.0 {
        space_avail = win.rect.width() - prefix.0;
    } else {
        space_avail = 0;
    }
    wprint_file_name(
        win.window,
        &file.file_name_as_string,
        Point::new(point.x, point.y + prefix.0 as i32),
        space_avail,
    );
}

pub fn wprint_entry_detailed(
    win: &window::JoshutoPanel,
    file: &structs::JoshutoDirEntry,
    prefix: (u32, &str),
    point: Point,
) {
    ncurses::waddstr(win.window, prefix.1);
    let point = Point::new(point.x, point.y + prefix.0 as i32);
    let mut space_avail: u32;
    if win.rect.width() >= prefix.0 {
        space_avail = win.rect.width() - prefix.0;
    } else {
        space_avail = 0;
    }

    if file.path.is_dir() {
    } else {
        let file_size_string = file_size_to_string(file.metadata.len as f64);
        if space_avail > file_size_string.len() {
            space_avail = space_avail - file_size_string.len();
            ncurses::mvwaddstr(win.window, point.y, space_avail as i32, &file_size_string);
        }
    }

    wprint_file_name(win.window, &file.file_name_as_string, point, space_avail);
}

pub fn wprint_file_mode(win: ncurses::WINDOW, file: &structs::JoshutoDirEntry) {
    use std::os::unix::fs::PermissionsExt;

    let mode = file.metadata.permissions.mode();

    ncurses::wattron(win, ncurses::COLOR_PAIR(6));
    ncurses::waddstr(win, unix::stringify_mode(mode).as_str());
    ncurses::wattroff(win, ncurses::COLOR_PAIR(6));
}

pub fn wprint_file_info(win: ncurses::WINDOW, file: &structs::JoshutoDirEntry) {
    use std::os::unix::fs::PermissionsExt;

    let mode = file.metadata.permissions.mode();

    let mtime_string = file_mtime_to_string(file.metadata.modified);
    ncurses::waddstr(win, &mtime_string);
    ncurses::waddch(win, ' ' as ncurses::chtype);

    if file.path.is_dir() {
        if mode >> 9 & libc::S_IFLNK >> 9 == mode >> 9 {
            if let Ok(path) = fs::read_link(&file.path) {
                ncurses::waddstr(win, " -> ");
                ncurses::waddstr(win, path.to_str().unwrap());
            }
        }
    } else {
        let file_size_string = file_size_to_string_detailed(file.metadata.len as f64);
        ncurses::waddstr(win, &file_size_string);
    }
}

pub fn redraw_tab_view(win: &window::JoshutoPanel, context: &JoshutoContext) {
    let tab_len = context.tabs.len();
    if tab_len == 1 {
        ncurses::werase(win.window);
    } else {
        ncurses::wmove(win.window, 0, 0);
        ncurses::wattron(win.window, ncurses::A_BOLD());
        ncurses::waddstr(
            win.window,
            format!("{} {}", context.curr_tab_index + 1, tab_len).as_str(),
        );
        ncurses::wattroff(win.window, ncurses::A_BOLD());
    }
    ncurses::wnoutrefresh(win.window);
}

pub fn draw_progress_bar(win: &window::JoshutoPanel, percentage: f32) {
    let cols: i32 = (win.rect.width() as f32 * percentage) as i32;
    ncurses::mvwchgat(
        win.window,
        0,
        0,
        cols,
        ncurses::A_STANDOUT(),
        theme_t.selection.colorpair,
    );
}

pub fn get_theme_attr(
    mut attr: ncurses::attr_t,
    entry: &structs::JoshutoDirEntry,
) -> ((u32, &str), ncurses::attr_t, i16) {
    use std::os::unix::fs::FileTypeExt;
    use std::os::unix::fs::PermissionsExt;

    let theme: &JoshutoColorTheme;
    let colorpair: i16;

    let file_type = &entry.metadata.file_type;
    if entry.selected {
        theme = &theme_t.selection;
        colorpair = theme_t.selection.colorpair;
    } else if file_type.is_dir() {
        theme = &theme_t.directory;
        colorpair = theme_t.directory.colorpair;
    } else if file_type.is_symlink() {
        theme = &theme_t.link;
        colorpair = theme_t.link.colorpair;
    } else if file_type.is_block_device() {
        theme = &theme_t.socket;
        colorpair = theme_t.link.colorpair;
    } else if file_type.is_char_device() {
        theme = &theme_t.socket;
        colorpair = theme_t.link.colorpair;
    } else if file_type.is_fifo() {
        theme = &theme_t.socket;
        colorpair = theme_t.link.colorpair;
    } else if file_type.is_socket() {
        theme = &theme_t.socket;
        colorpair = theme_t.link.colorpair;
    } else {
        let mode = entry.metadata.permissions.mode();
        if unix::is_executable(mode) {
            theme = &theme_t.executable;
            colorpair = theme_t.executable.colorpair;
        } else if let Some(ext) = entry.file_name_as_string.rfind('.') {
            let extension: &str = &entry.file_name_as_string[ext + 1..];
            if let Some(s) = theme_t.ext.get(extension) {
                theme = &s;
                colorpair = theme.colorpair;
            } else {
                theme = &theme_t.regular;
                colorpair = theme.colorpair;
            }
        } else {
            theme = &theme_t.regular;
            colorpair = theme.colorpair;
        }
    }

    if theme.bold {
        attr = attr | ncurses::A_BOLD();
    }
    if theme.underline {
        attr = attr | ncurses::A_UNDERLINE();
    }

    let mut prefix: (u32, &str) = (1, " ");
    if let Some(ref p1) = theme.prefix {
        if let Some(p2) = theme.prefixsize {
            prefix = (p2, &p1);
        }
    }

    (prefix, attr, colorpair)
}

fn file_size_to_string_detailed(mut file_size: f64) -> String {
    const FILE_UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "EB"];
    const CONV_RATE: f64 = 1024.0;

    let mut index = 0;
    while file_size > CONV_RATE {
        file_size = file_size / CONV_RATE;
        index += 1;
    }

    if file_size >= 1000.0 {
        format!("{:.0}{}", file_size, FILE_UNITS[index])
    } else if file_size >= 100.0 {
        format!(" {:.0}{}", file_size, FILE_UNITS[index])
    } else if file_size >= 10.0 {
        format!("{:.1}{}", file_size, FILE_UNITS[index])
    } else {
        format!("{:.2}{}", file_size, FILE_UNITS[index])
    }
}

fn file_mtime_to_string(mtime: time::SystemTime) -> String {
    const MTIME_FORMATTING: &str = "%Y-%m-%d %H:%M";

    let datetime: chrono::DateTime<chrono::offset::Utc> = mtime.into();
    datetime.format(MTIME_FORMATTING).to_string()
}

fn file_size_to_string(mut file_size: f64) -> String {
    const FILE_UNITS: [&str; 6] = ["B", "K", "M", "G", "T", "E"];
    const CONV_RATE: f64 = 1024.0;

    let mut index = 0;
    while file_size > CONV_RATE {
        file_size = file_size / CONV_RATE;
        index += 1;
    }

    if file_size >= 1000.0 {
        format!(" {:.0} {}", file_size, FILE_UNITS[index])
    } else if file_size >= 100.0 {
        format!(" {:.0} {}", file_size, FILE_UNITS[index])
    } else if file_size >= 10.0 {
        format!(" {:.1} {}", file_size, FILE_UNITS[index])
    } else {
        format!(" {:.2} {}", file_size, FILE_UNITS[index])
    }
}
