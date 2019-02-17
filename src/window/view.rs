extern crate ncurses;

use ui;
use utils::{Point, Rectangle};
use window::JoshutoPanel;

#[derive(Debug)]
pub struct JoshutoView {
    pub window_top: JoshutoPanel,
    pub window_bot: JoshutoPanel,
    pub window_left: JoshutoPanel,
    pub window_right: JoshutoPanel,
    pub window_mid: JoshutoPanel,
    pub window_tab: JoshutoPanel,
    pub window_ratio: (u32, u32, u32),
}

impl JoshutoView {
    pub fn new(window_ratio: (u32, u32, u32)) -> Self {
        let ratio_sum = window_ratio.0 + window_ratio.1 + window_ratio.2;

        let (width, height) = ui::get_terminal_dimensions();
        let (width, height) = (width as i32, height as i32);
        let terminal_divide = width / ratio_sum as i32;

        JoshutoView {
            window_top: JoshutoPanel::new(Rectangle::new(
                Point::origin(),
                Point::new(1, width - 5),
            )),
            window_bot: JoshutoPanel::new(Rectangle::new(
                Point::new(height - 1, 0),
                Point::new(height, width),
            )),
            window_left: JoshutoPanel::new(Rectangle::new(
                Point::new(1, 0),
                Point::new(height - 1, (terminal_divide * window_ratio.0 as i32) - 1),
            )),
            window_right: JoshutoPanel::new(Rectangle::new(
                Point::new(1, terminal_divide * window_ratio.2 as i32),
                Point::new(
                    height - 1,
                    2 * (terminal_divide * window_ratio.2 as i32) - 1,
                ),
            )),
            window_mid: JoshutoPanel::new(Rectangle::new(
                Point::new(1, terminal_divide * window_ratio.0 as i32),
                Point::new(
                    height - 1,
                    2 * (terminal_divide * window_ratio.1 as i32) - 1,
                ),
            )),
            window_tab: JoshutoPanel::new(Rectangle::new(
                Point::new(0, width - 5),
                Point::new(1, width),
            )),
            window_ratio,
        }
    }

    pub fn resize_views(&mut self) {
        let new_view = Self::new(self.window_ratio);

        self.window_top = new_view.window_top;
        self.window_bot = new_view.window_bot;
        self.window_left = new_view.window_left;
        self.window_right = new_view.window_right;
        self.window_mid = new_view.window_mid;
        self.window_tab = new_view.window_tab;
    }
}
