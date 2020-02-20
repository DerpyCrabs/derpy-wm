use std::io::{self, BufRead, Result};
use std::process::Command;

const GAP: usize = 5;

#[derive(Debug, Clone, PartialEq)]
enum EventType {
    EnterNotify,
    CreateNotify,
    DestroyNotify,
    MapNotify,
    Unknown,
}

#[derive(Debug, Clone)]
struct Event {
    window_id: String,
    event_type: EventType,
}

fn parse_event(ev_str: Result<String>) -> Event {
    let ev_str_parts: Vec<String> = ev_str
        .unwrap()
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect();
    let event_type = match ev_str_parts[0].as_str() {
        "ENTER" => EventType::EnterNotify,
        "CREATE" => EventType::CreateNotify,
        "DESTROY" => EventType::DestroyNotify,
        "MAP" => EventType::MapNotify,
        _ => EventType::Unknown,
    };
    Event {
        window_id: ev_str_parts[1].to_owned(),
        event_type,
    }
}

fn focus_window(window_id: &str) {
    Command::new("chwso").arg("-r").arg(window_id).status().ok();
    Command::new("wtf").arg(window_id).status().ok();
}

fn map_window(window_id: &str) {
    Command::new("mapw").arg("-m").arg(window_id).status().ok();
}

fn unmap_window(window_id: &str) {
    Command::new("mapw").arg("-u").arg(window_id).status().ok();
}

fn move_window(window_id: &str, x: usize, y: usize, w: usize, h: usize) {
    Command::new("wtp")
        .args(
            vec![x, y, w, h]
                .iter()
                .map(|i| format!("{}", i))
                .collect::<Vec<String>>(),
        )
        .arg(window_id)
        .status()
        .ok();
}

fn get_workspace_size(workspace_id: usize) -> (usize, usize) {
    (500, 300)
}

fn tile_workspace(workspace: &Vec<String>) {
    let (wsw, wsh) = get_workspace_size(0);
    let half_w = (wsw - 3 * GAP) / 2;
    let half_h = (wsh - 3 * GAP) / 2;
    let full_w = wsw - 2 * GAP;
    let full_h = wsh - 2 * GAP;

    match workspace.len() {
        0 => return,
        1 => {
            move_window(workspace[0].as_str(), GAP, GAP, full_w, full_h);
        }
        2 => {
            move_window(workspace[0].as_str(), GAP, GAP, half_w, full_h);
            move_window(workspace[1].as_str(), half_w + GAP * 2, GAP, half_w, full_h);
        }
        3 => {
            move_window(workspace[0].as_str(), GAP, GAP, half_w, full_h);
            move_window(workspace[1].as_str(), half_w + GAP * 2, GAP, half_w, half_h);
            move_window(
                workspace[2].as_str(),
                half_w + GAP * 2,
                half_h + GAP * 2,
                half_w,
                half_h,
            );
        }
        _ => {
            let mut hidden_windows = workspace.clone();
            let visible_windows = hidden_windows.split_off(workspace.len() - 4);
            hidden_windows
                .iter()
                .for_each(|wid| unmap_window(wid.as_str()));
            visible_windows
                .iter()
                .for_each(|wid| map_window(wid.as_str()));
            move_window(visible_windows[0].as_str(), GAP, GAP, half_w, half_h);
            move_window(
                visible_windows[1].as_str(),
                GAP,
                half_h + GAP * 2,
                half_w,
                half_h,
            );
            move_window(
                visible_windows[2].as_str(),
                half_w + GAP * 2,
                GAP,
                half_w,
                half_h,
            );
            move_window(
                visible_windows[3].as_str(),
                half_w + GAP * 2,
                half_h + GAP * 2,
                half_w,
                half_h,
            );
        }
    }
}
fn main() {
    let mut workspaces: Vec<Vec<String>> = vec![Vec::new()];
    let mut focused_workspace = 0;
    let mut last_event = Event {
        window_id: "0x0".to_string(),
        event_type: EventType::Unknown,
    };

    for event in io::stdin().lock().lines().map(parse_event) {
        println!("{:#?}", event);
        println!("{:#?}", workspaces[0]);
        match event.event_type {
            EventType::MapNotify => {
                if last_event.window_id == event.window_id
                    && last_event.event_type == EventType::CreateNotify
                {
                    focus_window(event.window_id.as_str());
                }
                tile_workspace(&workspaces[focused_workspace]);
            }
            EventType::CreateNotify => {
                workspaces[focused_workspace].push(event.window_id.clone());
            }
            EventType::DestroyNotify => {
                let mut was_in_focused_workspace = false;
                for (i, workspace) in workspaces.iter_mut().enumerate() {
                    if let Some(_) = workspace
                        .iter()
                        .find(|wid| wid == &event.window_id.as_str())
                    {
                        if i == focused_workspace {
                            was_in_focused_workspace = true;
                        }
                    }
                    workspace.retain(|wid| wid.as_str() != event.window_id);
                }

                if was_in_focused_workspace {
                    if let Some(window_id) = workspaces[focused_workspace].iter().last() {
                        focus_window(window_id.as_str());
                    }
                    tile_workspace(&workspaces[focused_workspace]);
                }
            }
            _ => (),
        };
        last_event = event;
    }
}
