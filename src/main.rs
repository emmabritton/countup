use chrono::{DateTime, Datelike, Utc};
use clap::{arg, command};
use color_eyre::Result;
use pixels_graphics_lib::prefs::WindowPreferences;
use pixels_graphics_lib::prelude::Positioning::{LeftTop, RightTop};
use pixels_graphics_lib::prelude::VirtualKeyCode::{Escape, Space};
use pixels_graphics_lib::prelude::*;

fn main() -> Result<()> {
    let matches = command!()
        .arg(arg!(-d --date <DATE> "Date to count from, format yyyy-mm-dd" ))
        .get_matches();

    let default = DateTime::parse_from_str("2022-11-25T00-00-00+0000", "%Y-%m-%dT%H-%M-%S%z")
        .expect("Default date invalid?")
        .with_timezone(&Utc);

    let (start, days) = match matches.get_one::<String>("date") {
        None => calc_days_since(default),
        Some(date) => {
            let date =
                DateTime::parse_from_str(&format!("{date}T00-00-00+0000"), "%Y-%m-%dT%H-%M-%S%z")
                    .expect("Invalid date")
                    .with_timezone(&Utc);
            if date > Utc::now() {
                panic!("Date must be in the past");
            } else {
                calc_days_since(date)
            }
        }
    };

    ui(
        days,
        format!("{:0>2}/{:0>2}/{}", start.day(), start.month(), start.year()),
        start,
    )
}

fn calc_days_since(date: DateTime<Utc>) -> (DateTime<Utc>, usize) {
    let diff = Utc::now() - date;
    (date, diff.num_days() as usize)
}

const COUNT_TIME_PER_YEAR: f64 = 1.0;
const COL_NUM: isize = 120;
const COL_PERIOD: isize = 128;

struct Countup {
    days: usize,
    start: String,
    start_date: DateTime<Utc>,
    should_exit: bool,
    current_days: usize,
    next_inc_speed: f64,
    next_inc: f64,
    diff_mode: bool
}

impl Countup {
    pub fn new(days: usize, start: String, start_date: DateTime<Utc>) -> Self {
        let f_days = days as f64;
        let next_inc_speed =
            ((f_days / 365.0) * COUNT_TIME_PER_YEAR).max(COUNT_TIME_PER_YEAR) / f_days;
        Self {
            start_date,
            days,
            start,
            should_exit: false,
            current_days: 0,
            next_inc_speed,
            next_inc: 0.0,
            diff_mode: false
        }
    }
}

fn ui(days: usize, start: String, start_date: DateTime<Utc>) -> Result<()> {
    let system = Box::new(Countup::new(days, start, start_date));
    run(270, 90, "Countup", system, Options::default())?;
    Ok(())
}

impl System for Countup {
    fn action_keys(&self) -> Vec<VirtualKeyCode> {
        vec![Escape, Space]
    }

    fn window_prefs(&self) -> Option<WindowPreferences> {
        Some(WindowPreferences::new("app", "emmabritton", "countup").unwrap())
    }

    fn update(&mut self, timing: &Timing) {
        if self.current_days < self.days {
            while self.next_inc < 0.0 && self.current_days < self.days {
                self.current_days += 1;
                self.next_inc += self.next_inc_speed;
            }
            self.next_inc -= timing.fixed_time_step;
        } else {
            let (_, day_count) = calc_days_since(self.start_date);
            if day_count != self.days {
                self.days = day_count;
                self.current_days = day_count;
            }
        }
    }

    fn render(&self, graphics: &mut Graphics) {
        if self.diff_mode {
            render_diff(graphics, self.current_days, &self.start)
        } else {
            render_split(graphics, self.current_days, &self.start);
        }
    }

    fn on_key_pressed(&mut self, keys: Vec<VirtualKeyCode>) {
        if keys.contains(&Escape) {
            self.should_exit = true
        } else if keys.contains(&Space) {
            self.current_days = 0;
            self.diff_mode= !self.diff_mode;
        }
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }
}

fn render_split(graphics: &mut Graphics, current_days: usize, start: &str) {
    graphics.clear(DARK_GRAY);
    graphics.draw_text(
        &format!("Since {} it's been", start),
        Px(4, 4),
        (LIGHT_GRAY, Large),
    );
    let years = current_days / 365;
    let remaining = current_days - (years * 365);
    let months = remaining / 28;
    let remaining = remaining - (months * 28);
    let days = remaining as usize;
    let months = months as usize;
    let years = years as usize;
    graphics.draw_text(
        &format!("{years}"),
        Px(COL_NUM, 24),
        (WHITE, Large, RightTop),
    );
    graphics.draw_text("YEARS", Px(COL_PERIOD, 24), (LIGHT_GRAY, Large, LeftTop));
    graphics.draw_text(
        &format!("{months}"),
        Px(COL_NUM, 40),
        (WHITE, Large, RightTop),
    );
    graphics.draw_text("MONTHS", Px(COL_PERIOD, 40), (LIGHT_GRAY, Large, LeftTop));
    graphics.draw_text(
        &format!("{days}"),
        Px(COL_NUM, 56),
        (WHITE, Large, RightTop),
    );
    graphics.draw_text("DAYS", Px(COL_PERIOD, 56), (LIGHT_GRAY, Large, LeftTop));
}

fn render_diff(graphics: &mut Graphics, current_days: usize, start: &str) {
    graphics.clear(DARK_GRAY);
    graphics.draw_text(
        &format!("Since {} it's been", start),
        Px(4, 4),
        (LIGHT_GRAY, Large),
    );
    let weeks = current_days / 7;
    let months = current_days / 28;
    let years = current_days / 365;
    graphics.draw_text(
        &format!("{current_days}"),
        Px(COL_NUM, 24),
        (WHITE, Large, RightTop),
    );
    graphics.draw_text("DAYS", Px(COL_PERIOD, 24), (LIGHT_GRAY, Large, LeftTop));
    graphics.draw_text("or", Px(COL_PERIOD + 42, 29), (LIGHT_GRAY, Small, LeftTop));
    graphics.draw_text(
        &format!("{weeks}"),
        Px(COL_NUM, 40),
        (WHITE, Large, RightTop),
    );
    graphics.draw_text("WEEKS", Px(COL_PERIOD, 40), (LIGHT_GRAY, Large, LeftTop));
    graphics.draw_text("or", Px(COL_PERIOD + 52, 45), (LIGHT_GRAY, Small, LeftTop));
    graphics.draw_text(
        &format!("{months}"),
        Px(COL_NUM, 56),
        (WHITE, Large, RightTop),
    );
    graphics.draw_text("MONTHS", Px(COL_PERIOD, 56), (LIGHT_GRAY, Large, LeftTop));
    graphics.draw_text("or", Px(COL_PERIOD + 62, 61), (LIGHT_GRAY, Small, LeftTop));
    graphics.draw_text(
        &format!("{years}"),
        Px(COL_NUM, 72),
        (WHITE, Large, RightTop),
    );
    graphics.draw_text("YEARS", Px(COL_PERIOD, 72), (LIGHT_GRAY, Large, LeftTop));
}