use chrono::{Local, Timelike};
use std::thread;
use std::time::Duration;

fn current_time() -> String {
    let now = Local::now();

    format!("{:02}:{:02}", now.hour(), now.minute())
}

fn should_redraw(old_time: &str, new_time: &str) -> bool {
    old_time != new_time
}

fn main() {
    let mut last_time = String::from("");

    loop {
        let time = current_time();

        if should_redraw(&last_time, &time) {
            println!("Novy cas: {}", time);

            last_time = time;
        }

        thread::sleep(Duration::from_secs(1));
    }
}