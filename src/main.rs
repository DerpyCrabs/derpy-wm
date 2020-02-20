use std::io::{self, BufRead, Result};
use std::process::Command;

const GAP: usize = 5;

#[derive(Debug, Clone, PartialEq)]
enum WindowEventType {
    EnterNotify,
    CreateNotify,
    DestroyNotify,
    MapNotify,
    Unknown,
}

#[derive(Debug, Clone)]
enum WorkspaceEventType {
    MoveWindow,
    Focus,
}

#[derive(Debug, Clone)]
struct WindowEvent {
    window_id: String,
    event_type: WindowEventType,
}

#[derive(Debug, Clone)]
struct WorkspaceEvent {
    workspace: usize,
    event_type: WorkspaceEventType,
}

#[derive(Debug, Clone)]
enum Event {
    Window(WindowEvent),
    Workspace(WorkspaceEvent),
    Unknown,
}

fn parse_event(ev_str: Result<String>) -> Event {
    let ev_str_parts: Vec<String> = ev_str
        .unwrap()
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect();
    if ["ENTER", "CREATE", "DESTROY", "MAP"].contains(&ev_str_parts[0].as_str()) {
        let event_type = match ev_str_parts[0].as_str() {
            "ENTER" => WindowEventType::EnterNotify,
            "CREATE" => WindowEventType::CreateNotify,
            "DESTROY" => WindowEventType::DestroyNotify,
            "MAP" => WindowEventType::MapNotify,
            _ => unreachable!(),
        };
        Event::Window(WindowEvent {
            window_id: ev_str_parts[1].to_owned(),
            event_type,
        })
    } else if ["WS_FOCUS", "WS_MOVE"].contains(&ev_str_parts[0].as_str()) {
        let event_type = match ev_str_parts[0].as_str() {
            "WS_FOCUS" => WorkspaceEventType::Focus,
            "WS_MOVE" => WorkspaceEventType::MoveWindow,
            _ => unreachable!(),
        };
        Event::Workspace(WorkspaceEvent {
            workspace: ev_str_parts[1].parse().expect("Invalid workspace event"),
            event_type,
        })
    } else {
        Event::Unknown
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

fn move_window(window_id: impl Into<String>, x: usize, y: usize, w: usize, h: usize) {
    Command::new("wtp")
        .args(
            vec![x, y, w, h]
                .iter()
                .map(|i| format!("{}", i))
                .collect::<Vec<String>>(),
        )
        .arg(window_id.into())
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
            move_window(&workspace[0], GAP, GAP, full_w, full_h);
        }
        2 => {
            move_window(&workspace[0], GAP, GAP, half_w, full_h);
            move_window(&workspace[1], half_w + GAP * 2, GAP, half_w, full_h);
        }
        3 => {
            move_window(&workspace[0], GAP, GAP, half_w, full_h);
            move_window(&workspace[1], half_w + GAP * 2, GAP, half_w, half_h);
            move_window(
                &workspace[2],
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
            move_window(&visible_windows[0], GAP, GAP, half_w, half_h);
            move_window(&visible_windows[1], GAP, half_h + GAP * 2, half_w, half_h);
            move_window(&visible_windows[2], half_w + GAP * 2, GAP, half_w, half_h);
            move_window(
                &visible_windows[3],
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
    let mut last_event = Event::Unknown;

    for event in io::stdin().lock().lines().map(parse_event) {
        let event_clone = event.clone();
        println!("{:#?}", event);
        println!("{:#?}", workspaces[0]);
        match event {
            Event::Window(event) => match event.event_type {
                WindowEventType::MapNotify => {
                    if let Event::Window(last_event) = last_event {
                        if last_event.window_id == event.window_id
                            && last_event.event_type == WindowEventType::CreateNotify
                        {
                            focus_window(event.window_id.as_str());
                        }
                    }
                    tile_workspace(&workspaces[focused_workspace]);
                }
                WindowEventType::CreateNotify => {
                    workspaces[focused_workspace].push(event.window_id);
                }
                WindowEventType::DestroyNotify => {
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
            },
            Event::Workspace(event) => {}
            Event::Unknown => {}
        }
        last_event = event_clone;
    }
}
