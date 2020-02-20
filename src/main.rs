use std::io::{self, BufRead, Result};
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
enum EventType {
  EnterNotify,
  CreateNotify,
  DestroyNotify,
  MapNotify,
  Unknown
}

#[derive(Debug, Clone)]
struct Event {
  window_id: String,
  event_type: EventType,
}

fn parse_event(ev_str: Result<String>) -> Event {
  let ev_str_parts: Vec<String> = ev_str.unwrap().split_whitespace().map(ToOwned::to_owned).collect();
  let event_type = match ev_str_parts[0].as_str() {
    "ENTER" => EventType::EnterNotify,
    "CREATE" => EventType::CreateNotify,
    "DESTROY" => EventType::DestroyNotify,
    "MAP" => EventType::MapNotify,
    _ => EventType::Unknown
  };
  Event {window_id: ev_str_parts[1].to_owned(), event_type}
}

fn focus_window(window_id: &str) {
  Command::new("chwso").arg("-r").arg(window_id).status().ok();
  Command::new("wtf").arg(window_id).status().ok();
}
fn map_window(window_id: &str) {
  Command::new("map").arg("-m").arg(window_id).status().ok();
}
fn main() {
  let mut workspaces: Vec<Vec<String>> = vec![Vec::new()];
  let mut focused_workspace = 0;
  let mut last_event = Event { window_id: "0x0".to_string(), event_type: EventType::Unknown };
  
  for event in io::stdin().lock().lines().map(parse_event) {
    println!("{:#?}", event);
    println!("{:#?}", workspaces[0]);
    match event.event_type {
      EventType::MapNotify => {
        if last_event.window_id == event.window_id && last_event.event_type == EventType::CreateNotify {
          focus_window(event.window_id.as_str());
        }
      },
      EventType::CreateNotify => {
        workspaces[focused_workspace].push(event.window_id.clone());
      },
      EventType::DestroyNotify => {
        for workspace in &mut workspaces {
          workspace.retain(|wid| wid.as_str() != event.window_id);
        }
        if let Some(window_id) = workspaces[focused_workspace].iter().last() {
          focus_window(window_id.as_str());
        }
      },
      _ => ()
    };
    last_event = event;
  }
}
