extern crate ncurses;

use std::collections::HashMap;
use std::time;

use commands;
use commands::{CommandKeybind, JoshutoCommand};
use config;
use context::JoshutoContext;
use preview;
use ui;
use utils::{Point, Rectangle};
use window::JoshutoPanel;

fn recurse_get_keycommand<'a>(
    keymap: &'a HashMap<i32, CommandKeybind>,
) -> Option<&Box<dyn JoshutoCommand>> {
    let (width, height) = ui::get_terminal_dimensions();
    ncurses::timeout(-1);

    let ch: i32;
    {
        let keymap_len = keymap.len();
        let win = JoshutoPanel::new(Rectangle::new(
            Point::new(height as i32 - keymap.len() as i32 - 2, 0),
            Point::new(height as i32 - 2, width as i32),
        ));

        let mut display_vec: Vec<String> = Vec::with_capacity(keymap_len);
        for (key, val) in keymap {
            display_vec.push(format!("  {}\t{}", *key as u8 as char, val));
        }
        display_vec.sort();

        win.move_to_top();
        ui::display_options(&win, &display_vec);
        ncurses::doupdate();

        ch = ncurses::wgetch(win.window);
    }
    ncurses::doupdate();

    if ch == config::keymap::ESCAPE {
        return None;
    }

    match keymap.get(&ch) {
        Some(CommandKeybind::CompositeKeybind(m)) => recurse_get_keycommand(&m),
        Some(CommandKeybind::SimpleKeybind(s)) => Some(s),
        _ => None,
    }
}

fn process_threads(context: &mut JoshutoContext) {
    let thread_wait_duration: time::Duration = time::Duration::from_millis(100);

    let mut i: u32 = 0;
    while i < context.threads.len() {
        match &context.threads[i].recv_timeout(&thread_wait_duration) {
            Ok(progress_info) => {
                if progress_info.bytes_finished == progress_info.total_bytes {
                    ncurses::werase(context.views.window_bot.window);
                    let thread = context.threads.swap_remove(i);
                    thread.handle.join().unwrap();
                    let (tab_src, tab_dest) = (thread.tab_src, thread.tab_dest);
                    if tab_src < context.tabs.len() {
                        context.tabs[tab_src].reload_contents(&context.config_t.sort_type);
                        if tab_src == context.curr_tab_index {
                            context.tabs[tab_src].refresh(
                                &context.views,
                                &context.config_t,
                                &context.username,
                                &context.hostname,
                            );
                        }
                    }
                    if tab_dest != tab_src && tab_dest < context.tabs.len() {
                        context.tabs[tab_dest].reload_contents(&context.config_t.sort_type);
                        if tab_dest == context.curr_tab_index {
                            context.tabs[tab_dest].refresh(
                                &context.views,
                                &context.config_t,
                                &context.username,
                                &context.hostname,
                            );
                        }
                    }
                } else {
                    let percent = (progress_info.bytes_finished as f64
                        / progress_info.total_bytes as f64)
                        as f32;
                    ui::draw_progress_bar(&context.views.window_bot, percent);
                    ncurses::wnoutrefresh(context.views.window_bot.window);
                    i = i + 1;
                }
                ncurses::doupdate();
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                ncurses::werase(context.views.window_bot.window);
                let thread = context.threads.swap_remove(i);
                let (tab_src, tab_dest) = (thread.tab_src, thread.tab_dest);
                thread.handle.join().unwrap();
                if tab_src < context.tabs.len() {
                    context.tabs[tab_src].reload_contents(&context.config_t.sort_type);
                    if tab_src == context.curr_tab_index {
                        context.tabs[tab_src].refresh(
                            &context.views,
                            &context.config_t,
                            &context.username,
                            &context.hostname,
                        );
                    }
                }
                if tab_dest != tab_src && tab_dest < context.tabs.len() {
                    context.tabs[tab_dest].reload_contents(&context.config_t.sort_type);
                    if tab_dest == context.curr_tab_index {
                        context.tabs[tab_dest].refresh(
                            &context.views,
                            &context.config_t,
                            &context.username,
                            &context.hostname,
                        );
                    }
                }
                ncurses::doupdate();
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                i = i + 1;
            }
        }
    }
}

fn resize_handler(context: &mut JoshutoContext) {
    ui::redraw_tab_view(&context.views.window_tab, &context);
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
}

#[allow(unreachable_code)]
pub fn run(config_t: config::JoshutoConfig, keymap_t: config::JoshutoKeymap) {
    ui::init_ncurses();

    let mut context = JoshutoContext::new(config_t);
    commands::NewTab::new_tab(&mut context);
    ncurses::doupdate();

    loop {
        if context.threads.len() > 0 {
            ncurses::timeout(0);
            process_threads(&mut context);
        } else {
            ncurses::timeout(-1);
        }

        if let Some(ch) = ncurses::get_wch() {
            let ch = match ch {
                ncurses::WchResult::Char(s) => s as i32,
                ncurses::WchResult::KeyCode(s) => s,
            };

            if ch == ncurses::KEY_RESIZE {
                context.views.resize_views();
                resize_handler(&mut context);
                continue;
            }

            let keycommand: &std::boxed::Box<dyn JoshutoCommand>;

            match keymap_t.keymaps.get(&ch) {
                Some(CommandKeybind::CompositeKeybind(m)) => match recurse_get_keycommand(&m) {
                    Some(s) => {
                        keycommand = s;
                    }
                    None => continue,
                },
                Some(CommandKeybind::SimpleKeybind(s)) => {
                    keycommand = s;
                }
                None => {
                    continue;
                }
            }
            keycommand.execute(&mut context);
        }
    }
    ncurses::endwin();
}
