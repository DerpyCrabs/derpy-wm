mod event_handlers;
mod reconciler;

use ::derpywm::{parse_event, Config, Event};
use reconciler::{actualize_screen, WMState};

use event_handlers::*;
use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("Provide config path");
    }
    let config_file = std::fs::read_to_string(args[1].clone()).expect("Can't find config.toml");
    let config: Config = toml::from_str(config_file.as_str()).expect("Can't parse config file");

    let mut last_event = Event::Unknown;
    let mut last_state = WMState::new(config.workspaces);

    for event in io::stdin().lock().lines().map(parse_event) {
        let mut now = last_state.clone();
        let event_clone = event.clone();

        dbg!(event.clone());

        match event {
            Event::Window(event) => {
                if !handle_window_event(&mut now, event, &last_event) {
                    continue;
                }
            }
            Event::Workspace(event) => {
                if !handle_workspace_event(&mut now, event) {
                    continue;
                }
            }
            Event::Scratchpad(event) => {
                if !handle_scratchpad_event(&mut now, event) {
                    continue;
                }
            }
            Event::Unknown => continue,
        }

        now.focused_window = actualize_screen(&last_state, &now, &config);
        last_event = event_clone;
        last_state = now;
    }
}
