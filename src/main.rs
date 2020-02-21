use std::io::{self, BufRead, Result};
use std::process::Command;

const GAP: usize = 5;
const WORKSPACES: usize = 3;

#[derive(Debug, Clone, PartialEq)]
enum WindowEventType {
    EnterNotify,
    CreateNotify,
    DestroyNotify,
    MapNotify,
}

#[derive(Debug, Clone)]
enum WorkspaceEventType {
    MoveWindow,
    Focus,
    Cycle,
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
    } else if ["WS_FOCUS", "WS_MOVE", "WS_CYCLE"].contains(&ev_str_parts[0].as_str()) {
        let event_type = match ev_str_parts[0].as_str() {
            "WS_FOCUS" => WorkspaceEventType::Focus,
            "WS_MOVE" => WorkspaceEventType::MoveWindow,
            "WS_CYCLE" => {
                return Event::Workspace(WorkspaceEvent {
                    workspace: 0,
                    event_type: WorkspaceEventType::Cycle,
                })
            }
            _ => unreachable!(),
        };
        let workspace: usize = ev_str_parts[1].parse().expect("Invalid workspace event");
        if workspace > 0 && workspace <= WORKSPACES {
            Event::Workspace(WorkspaceEvent {
                workspace: workspace - 1,
                event_type,
            })
        } else {
            Event::Unknown
        }
    } else {
        Event::Unknown
    }
}

fn focus_window(window_id: &str) {
    Command::new("wtf").arg(window_id).status().ok();
}

fn focused_window() -> Option<String> {
    let output = Command::new("pfw")
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .ok();
    if let Some(wid) = output {
        if wid != "" {
            return Some(wid);
        }
    }
    None
}

fn map_window(window_id: impl Into<String>) {
    Command::new("mapw")
        .arg("-m")
        .arg(window_id.into())
        .status()
        .ok();
}

fn unmap_window(window_id: impl Into<String>) {
    Command::new("mapw")
        .arg("-u")
        .arg(window_id.into())
        .status()
        .ok();
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

    match workspace.len() {
        0 => return,
        1 => {
            let full_w = wsw - 2 * GAP;
            let full_h = wsh - 2 * GAP;
            move_window(&workspace[0], GAP, GAP, full_w, full_h);
        }
        n => {
            let half_w = (wsw - 3 * GAP) / 2;
            let left_n = n / 2;
            let right_n = n - left_n;
            let left_h = (wsh - (left_n + 1) * GAP) / left_n;
            let right_h = (wsh - (right_n + 1) * GAP) / right_n;
            let mut left_strip = workspace.clone();
            let right_strip = left_strip.split_off(left_n);

            for (i, wid) in left_strip.iter().enumerate() {
                move_window(wid, GAP, GAP * (i + 1) + left_h * i, half_w, left_h);
            }
            for (i, wid) in right_strip.iter().enumerate() {
                move_window(
                    wid,
                    half_w + GAP * 2,
                    GAP * (i + 1) + right_h * i,
                    half_w,
                    right_h,
                );
            }
        }
    }
}

fn main() {
    let mut workspaces: Vec<Vec<String>> = vec![Vec::new(); WORKSPACES];
    let mut focused_workspace = 0;
    let mut last_event = Event::Unknown;

    for event in io::stdin().lock().lines().map(parse_event) {
        let event_clone = event.clone();
        println!("{:#?}", event);
        println!("{:#?}", workspaces);
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
            Event::Workspace(event) => match event.event_type {
                WorkspaceEventType::Focus => {
                    if event.workspace != focused_workspace {
                        workspaces[focused_workspace].iter().for_each(unmap_window);
                        focused_workspace = event.workspace;
                        workspaces[event.workspace].iter().for_each(map_window);
                    }
                }
                WorkspaceEventType::MoveWindow => {
                    if event.workspace != focused_workspace {
                        if let Some(focused_wid) = focused_window() {
                            unmap_window(focused_wid.as_str());
                            workspaces[focused_workspace]
                                .retain(|wid| wid.as_str() != focused_wid.as_str());
                            workspaces[event.workspace].push(focused_wid);
                            tile_workspace(&workspaces[focused_workspace]);
                            if let Some(window_id) = workspaces[focused_workspace].iter().last() {
                                focus_window(window_id.as_str());
                            }
                        }
                    }
                }
                WorkspaceEventType::Cycle => {
                    if workspaces[focused_workspace].len() > 0 {
                        workspaces[focused_workspace].rotate_right(1);
                        tile_workspace(&workspaces[focused_workspace]);
                    }
                }
            },
            Event::Unknown => {}
        }
        last_event = event_clone;
    }
}
